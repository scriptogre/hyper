//! Lowering pass: hyper AST → Ruff Python AST (`ModModule`).
//!
//! This is where the intelligence that used to live in the string-based code
//! generator moves. Instead of concatenating Python text, we build a real
//! Python AST that downstream plugins transform and a source-map-aware printer
//! renders.
//!
//! The lowering is intentionally "dumb" about program-level concerns that the
//! plugins own (async-ness, slot parameters, helper imports, mutable-default
//! guards, `**kwargs` spreads). It produces the structural skeleton — user
//! imports, the `@html` decorator, the function signature with its declared
//! parameters, and the lowered body — and lets the plugin passes refine it.
//!
//! ## Status
//!
//! Phase 1 (this module): outer structure + a subset of body nodes. Body node
//! kinds that are not yet lowered to real Python AST are emitted as transitional
//! string-constant `yield`s (clearly marked) so the pipeline stays whole while
//! Phase 2 fills them in. The existing string-based generator remains the
//! default; this path is exercised by unit tests via [`crate::compile_via_ast`].

pub mod builders;
pub mod render;

use ruff_python_ast::{self as ast, Stmt};

use crate::ast::{
    Ast, DecoratorNode, DefinitionNode, ForNode, IfNode, MatchNode, Node, ParameterNode, TryNode,
    WhileNode, WithNode,
};
use crate::error::CompileError;
use builders as b;

/// Convert a snake_case file stem into the PascalCase component name.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Partitioned top-level nodes of a hyper template.
struct Partition<'a> {
    imports: Vec<&'a crate::ast::ImportNode>,
    params: Vec<&'a ParameterNode>,
    /// Orphan decorators applied to the outer template function.
    decorators: Vec<&'a crate::ast::DecoratorNode>,
    body: Vec<&'a Node>,
}

/// Split the flat top-level node list into header (imports/params/decorators)
/// and body, mirroring the partitioning the string generator performs.
fn partition(nodes: &[Node]) -> Partition<'_> {
    let mut imports = Vec::new();
    let mut params = Vec::new();
    let mut decorators = Vec::new();
    let mut body = Vec::new();

    // A decorator that is immediately followed (modulo comments/blank text) by a
    // definition belongs to that definition and stays in the body; otherwise it
    // is an orphan decorator applied to the outer `@html` function.
    let mut decorator_leads_to_def = vec![false; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        if matches!(node, Node::Decorator(_)) {
            for next in &nodes[i + 1..] {
                match next {
                    Node::Decorator(_) | Node::Comment(_) => continue,
                    Node::Text(t) if t.content.trim().is_empty() => continue,
                    Node::Definition(_) => {
                        decorator_leads_to_def[i] = true;
                        break;
                    }
                    _ => break,
                }
            }
        }
    }

    for (i, node) in nodes.iter().enumerate() {
        match node {
            Node::Parameter(p) => params.push(p),
            Node::Import(im) => imports.push(im),
            Node::Decorator(d) if !decorator_leads_to_def[i] => decorators.push(d),
            _ => body.push(node),
        }
    }

    Partition {
        imports,
        params,
        decorators,
        body,
    }
}

/// Lower a parameter declaration into a keyword-only function parameter.
fn lower_parameter(param: &ParameterNode) -> Result<ast::ParameterWithDefault, CompileError> {
    let range = b::span_range(param.span);
    let annotation = match &param.type_hint {
        Some(hint) => Some(b::parse_expr(hint)?),
        None => None,
    };
    let default = match &param.default {
        Some(def) => Some(b::parse_expr(def)?),
        None => None,
    };
    Ok(b::kwonly_param(&param.name, range, annotation, default))
}

/// Lower a sequence of body nodes into a flat list of Python statements,
/// grouping leading decorators with the definition they apply to.
fn lower_seq(nodes: &[&Node]) -> Result<Vec<Stmt>, CompileError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < nodes.len() {
        // Combine a run of adjacent text/expression/element nodes into a single
        // yielded (f-)string, mirroring the string generator's grouping.
        if render::is_combinable(nodes[i]) {
            let mut j = i + 1;
            while j < nodes.len() && render::is_combinable(nodes[j]) {
                j += 1;
            }
            if let Some(stmt) = lower_combined_run(&nodes[i..j])? {
                out.push(stmt);
            }
            i = j;
            continue;
        }
        if matches!(nodes[i], Node::Decorator(_)) {
            // Collect a run of decorators (and intervening comments/blank text)
            // and attach them to the definition that follows.
            let mut decorators: Vec<&DecoratorNode> = Vec::new();
            let mut j = i;
            while j < nodes.len() {
                match nodes[j] {
                    Node::Decorator(d) => {
                        decorators.push(d);
                        j += 1;
                    }
                    Node::Comment(_) => j += 1,
                    Node::Text(t) if t.content.trim().is_empty() => j += 1,
                    _ => break,
                }
            }
            if let Some(Node::Definition(def)) = nodes.get(j).copied() {
                out.extend(lower_definition(&decorators, def)?);
                i = j + 1;
                continue;
            }
            // No definition follows; emit the decorators verbatim (rare).
            for d in &decorators {
                out.extend(b::parse_stmts(&d.decorator)?);
            }
            i = j;
            continue;
        }
        out.extend(lower_node(nodes[i])?);
        i += 1;
    }
    Ok(out)
}

/// Lower a combinable run of content nodes into a single yielded (f-)string.
///
/// Returns `None` for an all-whitespace run, matching the string generator,
/// which emits no `yield` for structurally-empty content.
fn lower_combined_run(nodes: &[&Node]) -> Result<Option<Stmt>, CompileError> {
    let rendered = render::render_run(nodes);
    if rendered.content.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(yield_str_source(&rendered.content, rendered.has_expr)?))
}

/// Build a `yield "<content>"` / `yield f"<content>"` statement by parsing the
/// (f-)string source into a real Ruff string/f-string expression.
fn yield_str_source(content: &str, has_expr: bool) -> Result<Stmt, CompileError> {
    let source = if has_expr {
        format!("f\"\"\"{content}\"\"\"")
    } else {
        format!("\"\"\"{content}\"\"\"")
    };
    let expr = b::parse_expr(&source)?;
    Ok(b::expr_stmt(b::yield_expr(expr), b::SENTINEL))
}

/// Lower a non-combinable element (one whose children include a component, slot,
/// or control flow): yield the open tag, lower the children, yield the close tag.
fn lower_element(el: &crate::ast::ElementNode) -> Result<Vec<Stmt>, CompileError> {
    let open = render::render_open_tag(el);
    let mut stmts = vec![yield_str_source(&open.content, open.has_expr)?];
    if !el.self_closing {
        let refs: Vec<&Node> = el.children.iter().collect();
        stmts.extend(lower_seq(&refs)?);
        stmts.push(yield_str_source(&format!("</{}>", el.tag), false)?);
    }
    Ok(stmts)
}

/// Lower the children of a control-flow branch, defaulting to `pass` when empty
/// so the produced suite is always valid Python.
fn lower_children(nodes: &[Node]) -> Result<Vec<Stmt>, CompileError> {
    let refs: Vec<&Node> = nodes.iter().collect();
    let mut stmts = lower_seq(&refs)?;
    if stmts.is_empty() {
        stmts.push(b::pass_stmt());
    }
    Ok(stmts)
}

/// Lower a single hyper body node into zero or more Python statements.
fn lower_node(node: &Node) -> Result<Vec<Stmt>, CompileError> {
    match node {
        // Blank/structural-only text between header items produces nothing.
        Node::Text(t) if t.content.is_empty() => Ok(vec![]),

        // A Python statement parses straight through into real statements.
        Node::Statement(s) => b::parse_stmts(&s.stmt),

        // A bare expression yields its (optionally escaped) value.
        Node::Expression(e) => {
            let expr = b::parse_expr(&e.expr)?;
            let value = if e.escape {
                let escape_fn = b::name_expr("escape", b::SENTINEL);
                b::call(escape_fn, vec![expr], vec![], b::span_range(e.span))
            } else {
                expr
            };
            Ok(vec![b::expr_stmt(b::yield_expr(value), b::span_range(e.span))])
        }

        // Comments carry no runtime effect in the lowered Python.
        Node::Comment(_) => Ok(vec![]),

        // Control flow: reconstruct the header as a Python skeleton, parse it
        // with Ruff, then graft the recursively-lowered children into the body.
        Node::If(n) => lower_if(n),
        Node::For(n) => lower_for(n),
        Node::While(n) => lower_while(n),
        Node::With(n) => lower_with(n),
        Node::Match(n) => lower_match(n),
        Node::Try(n) => lower_try(n),

        // A bare definition (decorators are grouped in by `lower_seq`).
        Node::Definition(def) => lower_definition(&[], def),

        // A fragment is just its children with no wrapping element.
        Node::Fragment(f) => {
            let refs: Vec<&Node> = f.children.iter().collect();
            lower_seq(&refs)
        }

        // A non-combinable element (has a component/slot/control-flow child):
        // yield the open tag, lower children, yield the close tag.
        Node::Element(el) => lower_element(el),

        // Component invocation → `yield from Name(...)`.
        Node::Component(c) => lower_component(c),

        // Slot placeholder → conditional `yield from` with fallback.
        Node::Slot(s) => lower_slot(s),

        // Transitional: HTML-producing kinds (text, elements, components,
        // fragments, slots) still emit string-constant yields pending the
        // f-string lowering step.
        other => Ok(vec![transitional_yield(other)]),
    }
}

/// Strip the trailing block colon that the hyper parser keeps on control-flow
/// header strings (`"x > 1:"` → `"x > 1"`), so we can append our own.
fn header(s: &str) -> &str {
    s.trim_end_matches(':').trim()
}

/// Parse a reconstructed-header skeleton and return its single statement.
fn parse_one(src: &str) -> Result<Stmt, CompileError> {
    b::parse_stmts(src)?
        .into_iter()
        .next()
        .ok_or_else(|| CompileError::Generate(format!("empty skeleton parse: `{src}`")))
}

fn lower_if(n: &IfNode) -> Result<Vec<Stmt>, CompileError> {
    let mut src = format!("if {}:\n    pass\n", header(&n.condition));
    for (cond, _, _) in &n.elif_branches {
        src.push_str(&format!("elif {}:\n    pass\n", header(cond)));
    }
    if n.else_branch.is_some() {
        src.push_str("else:\n    pass\n");
    }
    let mut stmt = parse_one(&src)?;
    if let Stmt::If(if_stmt) = &mut stmt {
        if_stmt.body = lower_children(&n.then_branch)?.into_iter().collect();
        let mut elif_idx = 0;
        for clause in &mut if_stmt.elif_else_clauses {
            if clause.test.is_some() {
                clause.body = lower_children(&n.elif_branches[elif_idx].2)?.into_iter().collect();
                elif_idx += 1;
            } else if let Some(else_branch) = &n.else_branch {
                clause.body = lower_children(else_branch)?.into_iter().collect();
            }
        }
    }
    Ok(vec![stmt])
}

fn lower_for(n: &ForNode) -> Result<Vec<Stmt>, CompileError> {
    let prefix = if n.is_async { "async " } else { "" };
    let src = format!("{prefix}for {} in {}:\n    pass\n", n.binding.trim(), header(&n.iterable));
    let mut stmt = parse_one(&src)?;
    if let Stmt::For(for_stmt) = &mut stmt {
        for_stmt.body = lower_children(&n.body)?.into_iter().collect();
    }
    Ok(vec![stmt])
}

fn lower_while(n: &WhileNode) -> Result<Vec<Stmt>, CompileError> {
    let src = format!("while {}:\n    pass\n", header(&n.condition));
    let mut stmt = parse_one(&src)?;
    if let Stmt::While(while_stmt) = &mut stmt {
        while_stmt.body = lower_children(&n.body)?.into_iter().collect();
    }
    Ok(vec![stmt])
}

fn lower_with(n: &WithNode) -> Result<Vec<Stmt>, CompileError> {
    let prefix = if n.is_async { "async " } else { "" };
    let src = format!("{prefix}with {}:\n    pass\n", header(&n.items));
    let mut stmt = parse_one(&src)?;
    if let Stmt::With(with_stmt) = &mut stmt {
        with_stmt.body = lower_children(&n.body)?.into_iter().collect();
    }
    Ok(vec![stmt])
}

fn lower_match(n: &MatchNode) -> Result<Vec<Stmt>, CompileError> {
    let mut src = format!("match {}:\n", header(&n.expr));
    for case in &n.cases {
        src.push_str(&format!("    case {}:\n        pass\n", header(&case.pattern)));
    }
    let mut stmt = parse_one(&src)?;
    if let Stmt::Match(match_stmt) = &mut stmt {
        for (case_ast, case_node) in match_stmt.cases.iter_mut().zip(&n.cases) {
            case_ast.body = lower_children(&case_node.body)?.into_iter().collect();
        }
    }
    Ok(vec![stmt])
}

fn lower_try(n: &TryNode) -> Result<Vec<Stmt>, CompileError> {
    let mut src = String::from("try:\n    pass\n");
    for clause in &n.except_clauses {
        match &clause.exception {
            Some(exc) => src.push_str(&format!("except {}:\n    pass\n", header(exc))),
            None => src.push_str("except:\n    pass\n"),
        }
    }
    if n.else_clause.is_some() {
        src.push_str("else:\n    pass\n");
    }
    if n.finally_clause.is_some() {
        src.push_str("finally:\n    pass\n");
    }
    let mut stmt = parse_one(&src)?;
    if let Stmt::Try(try_stmt) = &mut stmt {
        try_stmt.body = lower_children(&n.body)?.into_iter().collect();
        for (handler, clause) in try_stmt.handlers.iter_mut().zip(&n.except_clauses) {
            let ast::ExceptHandler::ExceptHandler(h) = handler;
            h.body = lower_children(&clause.body)?.into_iter().collect();
        }
        if let Some(else_clause) = &n.else_clause {
            try_stmt.orelse = lower_children(else_clause)?.into_iter().collect();
        }
        if let Some(finally_clause) = &n.finally_clause {
            try_stmt.finalbody = lower_children(finally_clause)?.into_iter().collect();
        }
    }
    Ok(vec![stmt])
}

/// PascalCase component name → the inner default-slot generator function name,
/// e.g. `Inner` → `_inner_default_slot`. Mirrors the string generator.
fn component_to_func_name(name: &str) -> String {
    let mut result = String::from("_");
    let mut prev_was_separator = false;
    for (i, ch) in name.chars().enumerate() {
        if ch.is_alphanumeric() || ch == '_' {
            if ch.is_uppercase() && i > 0 && !prev_was_separator {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_was_separator = false;
        } else {
            if !prev_was_separator && i > 0 && !result.ends_with('_') {
                result.push('_');
            }
            prev_was_separator = true;
        }
    }
    while result.ends_with('_') && result.len() > 1 {
        result.pop();
    }
    result.push_str("_default_slot");
    result
}

/// Parameter name a slot binds to: default → `_default_slot`, named → `_<n>_slot`.
fn slot_param_name(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("_{n}_slot"),
        None => "_default_slot".to_string(),
    }
}

/// Lower a component invocation into `yield from Name(...)`. Components with
/// children get an inner default-slot generator passed as the first argument.
fn lower_component(c: &crate::ast::ComponentNode) -> Result<Vec<Stmt>, CompileError> {
    let kwargs = render::component_kwargs(&c.attributes);
    let name_range = b::span_range(c.name_span);

    if c.children.is_empty() {
        let call = b::parse_expr(&format!("{}({})", c.name, kwargs))?;
        return Ok(vec![b::expr_stmt(b::yield_from_expr(call), name_range)]);
    }

    // With children: emit `def _<name>_default_slot(): <children>` then call it.
    let func_name = component_to_func_name(&c.name);
    let body = lower_children(&c.children)?;
    let inner = b::function_def(&func_name, false, vec![], b::empty_parameters(), body);

    let mut args = format!("{func_name}()");
    if !kwargs.is_empty() {
        args.push_str(", ");
        args.push_str(&kwargs);
    }
    let call = b::parse_expr(&format!("{}({})", c.name, args))?;
    let yield_from = b::expr_stmt(b::yield_from_expr(call), name_range);
    Ok(vec![inner, yield_from])
}

/// Lower a slot placeholder into `if <slot> is not None: yield from <slot>`
/// with the fallback content as the `else` branch.
fn lower_slot(s: &crate::ast::SlotNode) -> Result<Vec<Stmt>, CompileError> {
    let slot_var = slot_param_name(s.name.as_deref());
    let has_fallback = !s.fallback.is_empty();

    let src = if has_fallback {
        format!("if {slot_var} is not None:\n    pass\nelse:\n    pass\n")
    } else {
        format!("if {slot_var} is not None:\n    pass\n")
    };
    let mut stmt = parse_one(&src)?;
    if let Stmt::If(if_stmt) = &mut stmt {
        let yield_from = b::expr_stmt(
            b::yield_from_expr(b::name_expr(&slot_var, b::SENTINEL)),
            b::SENTINEL,
        );
        if_stmt.body = std::iter::once(yield_from).collect();
        if has_fallback {
            for clause in &mut if_stmt.elif_else_clauses {
                clause.body = lower_children(&s.fallback)?.into_iter().collect();
            }
        }
    }
    Ok(vec![stmt])
}

/// Lower a function/class definition, reconstructing `decorators + signature`
/// as a skeleton and grafting the recursively-lowered body.
fn lower_definition(
    decorators: &[&DecoratorNode],
    def: &DefinitionNode,
) -> Result<Vec<Stmt>, CompileError> {
    let mut src = String::new();
    for d in decorators {
        src.push_str(d.decorator.trim_end());
        src.push('\n');
    }
    src.push_str(def.signature.trim_end());
    src.push_str("\n    pass\n");
    let mut stmt = parse_one(&src)?;
    match &mut stmt {
        Stmt::FunctionDef(f) => f.body = lower_children(&def.body)?.into_iter().collect(),
        Stmt::ClassDef(c) => c.body = lower_children(&def.body)?.into_iter().collect(),
        _ => {
            return Err(CompileError::Generate(format!(
                "definition signature did not parse to a def/class: `{}`",
                def.signature
            )));
        }
    }
    Ok(vec![stmt])
}

/// A placeholder `yield "<…>"` for a body node kind Phase 2 will lower properly.
fn transitional_yield(node: &Node) -> Stmt {
    let label = match node {
        Node::Text(_) => "text",
        Node::Element(_) => "element",
        Node::Component(_) => "component",
        Node::Slot(_) => "slot",
        Node::If(_) => "if",
        Node::For(_) => "for",
        Node::Match(_) => "match",
        Node::While(_) => "while",
        Node::With(_) => "with",
        Node::Try(_) => "try",
        Node::Definition(_) => "definition",
        _ => "node",
    };
    let placeholder = b::string_literal(&format!("__hyper_todo__:{label}"), b::SENTINEL);
    b::expr_stmt(b::yield_expr(placeholder), b::SENTINEL)
}

/// Lower a hyper [`Ast`] into a Ruff [`ast::ModModule`].
///
/// `function_name` is the (snake_case) file stem; it is PascalCased to form the
/// component function name.
pub fn lower(ast: &Ast, function_name: Option<&str>) -> Result<ast::ModModule, CompileError> {
    let part = partition(&ast.nodes);

    let mut module_body: Vec<Stmt> = Vec::new();

    // 1. User imports, parsed straight through.
    for import in &part.imports {
        module_body.extend(b::parse_stmts(&import.stmt)?);
    }

    // 2. The generated `from hyper import html` (helper plugin will extend this).
    module_body.push(b::import_from("hyper", &[("html", None)], b::SENTINEL));

    // 3. Function parameters (keyword-only, after the bare `*`).
    let mut parameters = b::empty_parameters();
    for param in &part.params {
        parameters.kwonlyargs.push(lower_parameter(param)?);
    }

    // 4. Function body.
    let mut func_body = lower_seq(&part.body)?;
    if func_body.is_empty() {
        func_body.push(b::pass_stmt());
    }

    // 5. Decorators: user orphan decorators first, then `@html`.
    let mut decorators: Vec<ast::Decorator> = Vec::new();
    for dec in &part.decorators {
        let text = dec.decorator.trim_start_matches('@');
        let expr = b::parse_expr(text)?;
        decorators.push(b::decorator(expr, b::span_range(dec.span)));
    }
    decorators.push(b::decorator(b::name_expr("html", b::SENTINEL), b::SENTINEL));

    let name = function_name.map(to_pascal_case).unwrap_or_else(|| "Render".to_string());

    module_body.push(b::function_def(
        &name,
        /* is_async */ false,
        decorators,
        parameters,
        func_body,
    ));

    Ok(b::module(module_body))
}

#[cfg(test)]
mod fixture_tests {
    use crate::{CompileOptions, compile, compile_via_ast};
    use std::path::Path;

    /// Every `.hyper` fixture the string pipeline compiles must also lower
    /// through the new pipeline into *valid, parseable* Python with no
    /// transitional placeholders left behind.
    #[test]
    fn lowers_all_fixtures_to_valid_python() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let mut checked = 0;
        let mut failures: Vec<String> = Vec::new();

        for dir in ["basic", "components"] {
            let dir = root.join(dir);
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("hyper") {
                    continue;
                }
                let src = std::fs::read_to_string(&path).unwrap();
                let stem = path.file_stem().unwrap().to_str().unwrap();

                // Only fixtures the string pipeline accepts are in scope.
                let opts = CompileOptions {
                    function_name: Some(stem.to_string()),
                    include_ranges: false,
                };
                if compile(&src, &opts).is_err() {
                    continue;
                }
                checked += 1;

                match compile_via_ast(&src, Some(stem)) {
                    Err(e) => failures.push(format!("{stem}: lowering error: {e}")),
                    Ok(out) if out.contains("__hyper_todo__") => {
                        failures.push(format!("{stem}: transitional placeholder remains"))
                    }
                    Ok(out) => {
                        if let Err(e) = ruff_python_parser::parse_module(&out) {
                            failures.push(format!("{stem}: output not valid Python: {e}"));
                        }
                    }
                }
            }
        }

        assert!(checked > 0, "no fixtures were checked");
        assert!(
            failures.is_empty(),
            "{}/{} fixtures failed to lower cleanly:\n{}",
            failures.len(),
            checked,
            failures.join("\n")
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::compile_via_ast;

    #[test]
    fn lowers_only_params() {
        let src = "name: str\ncount: int = 0\nitems: list\n\n---\n";
        let out = compile_via_ast(src, Some("only_params")).unwrap();
        println!("=== only_params ===\n{out}");
        assert!(out.contains("def OnlyParams"));
        assert!(out.contains("from hyper import html"));
    }

    #[test]
    fn lowers_imports_and_expressions() {
        let src = "from datetime import datetime\nimport json\n\nname: str\n\n---\n\n<p>{datetime.now().isoformat()}</p>\n";
        let out = compile_via_ast(src, Some("imports")).unwrap();
        println!("=== imports ===\n{out}");
        assert!(out.contains("from datetime import datetime"));
        assert!(out.contains("import json"));
    }
}

#[cfg(test)]
mod control_flow_tests {
    use crate::compile_via_ast;

    fn lower(src: &str) -> String {
        compile_via_ast(src, Some("t")).unwrap()
    }

    #[test]
    fn lowers_if_elif_else() {
        let out = lower("x: int\n\n---\n\nif x > 1:\n    <a>one</a>\nelif x > 0:\n    <b>zero</b>\nelse:\n    <c>neg</c>\nend\n");
        println!("{out}");
        assert!(out.contains("if x > 1:"));
        assert!(out.contains("elif x > 0:"));
        assert!(out.contains("else:"));
    }

    #[test]
    fn lowers_for_while_with_try_match() {
        let out = lower("items: list\n\n---\n\nfor i in items:\n    {i}\nend\nwhile items:\n    {items}\nend\nwith open('f') as fh:\n    {fh}\nend\ntry:\n    {items}\nexcept ValueError as e:\n    {e}\nfinally:\n    {items}\nend\nmatch items:\n    case []:\n        {items}\nend\n");
        println!("{out}");
        for needle in ["for i in items:", "while items:", "with open('f') as fh:", "try:", "except ValueError as e:", "finally:", "match items:", "case []:"] {
            assert!(out.contains(needle), "missing: {needle}\n{out}");
        }
    }

    #[test]
    fn lowers_definition_with_decorator() {
        let out = lower("---\n\n@staticmethod\ndef helper(x: int) -> int:\n    return x * 2\nend\n");
        println!("{out}");
        assert!(out.contains("@staticmethod"));
        assert!(out.contains("def helper(x: int) -> int:"));
        assert!(out.contains("return x * 2"));
    }
}
