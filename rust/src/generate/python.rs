use super::{
    CompileOptions, CompileResult, Generator, Language, Output, Segment,
    collect_component_attr_expr_spans, collect_expression_braces, convert_braces_to_utf16,
    html_segments_for_component, html_segments_for_element,
};
use crate::ast::python::{Alias, Code, Identifier, StmtImportFrom};
use crate::ast::*;
use crate::generate::print::{print_code, print_expr, print_import_from};
use crate::html;
use crate::lower::{code_span, helper_call, lower_interpolation, render_attr_call};
use crate::plugins::{DEFAULT_SLOT_PARAM, Helper, rename_reserved_keywords, slot_param_name};

/// Where a dynamic attribute's helper call lands in the f-string.
enum Scaffold<'a> {
    /// ` name="{<expr>}"`. Attribute name stays static, helper fills value slot.
    Value(&'a str),
    /// `{<expr>}`. Helper emits or omits attribute name and value.
    Whole,
}

pub struct PythonGenerator;

impl PythonGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Check if a list of nodes contains only whitespace/newline text (no real content)
    fn is_effectively_empty(&self, nodes: &[&Node]) -> bool {
        nodes.iter().all(|node| match node {
            Node::Text(text) => text.content.trim().is_empty(),
            Node::Fragment(fragment) => {
                let children: Vec<&Node> = fragment.children.iter().collect();
                self.is_effectively_empty(&children)
            }
            Node::Slot(slot) if slot.is_fill => {
                let fallback: Vec<&Node> = slot.fallback.iter().collect();
                self.is_effectively_empty(&fallback)
            }
            _ => false,
        })
    }

    /// Emit body nodes, or `pass` if the body is empty/whitespace-only
    fn emit_body_or_pass(&self, body: &[Node], output: &mut Output, indent: usize) {
        let refs: Vec<&Node> = body.iter().collect();
        if refs.is_empty() || self.is_effectively_empty(&refs) {
            self.indent(output, indent);
            output.push("pass");
            output.newline();
        } else {
            self.emit_nodes(&refs, output, indent);
        }
    }

    /// Check if a node can be combined into a string literal (not control flow)
    fn is_combinable(&self, node: &Node) -> bool {
        match node {
            Node::Text(_) | Node::Expression(_) => true,
            Node::Element(el) => {
                // Element is combinable only if all its children are combinable
                el.children.iter().all(|child| self.is_combinable(child))
            }
            _ => false, // Components, Slots, control flow, etc. are not combinable
        }
    }

    /// Emit a group of nodes, combining consecutive text/expression nodes into f-strings
    fn emit_nodes(&self, nodes: &[&Node], output: &mut Output, indent: usize) {
        let mut i = 0;
        while i < nodes.len() {
            // Check if this node and following nodes can be combined into a string
            let can_combine = self.is_combinable(nodes[i]);

            if can_combine {
                // Find the end of the combinable sequence
                let mut j = i + 1;
                while j < nodes.len() && self.is_combinable(nodes[j]) {
                    j += 1;
                }

                // Check if the next node is an inline comment (same source line as content)
                let trailing_comment = if j < nodes.len() {
                    if let Node::Comment(c) = nodes[j] {
                        if c.inline { Some(c) } else { None }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Emit combined nodes as a single string/f-string
                self.emit_combined_nodes(&nodes[i..j], output, indent, trailing_comment);
                i = j;
                if trailing_comment.is_some() {
                    i += 1; // skip the comment we already emitted inline
                }
            } else {
                // Emit control flow or other non-combinable nodes individually
                self.emit_node(nodes[i], output, indent);
                i += 1;
            }
        }
    }

    /// Check if a node or its descendants contain expressions
    fn node_has_expressions(&self, node: &Node) -> bool {
        match node {
            Node::Expression(_) => true,
            Node::Element(el) => {
                // Check if element has dynamic attributes or expression children
                el.attributes.iter().any(|attr| {
                    !matches!(
                        attr.kind,
                        AttributeKind::Static { .. }
                            | AttributeKind::Boolean { .. }
                            | AttributeKind::SlotAssignment { .. }
                    )
                }) || el
                    .children
                    .iter()
                    .any(|child| self.node_has_expressions(child))
            }
            _ => false,
        }
    }

    /// Emit consecutive text/expression/element nodes as a single yield statement.
    /// If trailing_comment is Some, the comment is appended inline after the closing `"""`.
    ///
    /// Uses a two-phase approach:
    ///   Phase 1: Emit to a temp buffer for content analysis (segments discarded).
    ///   Phase 2: Emit to real output with skip/dedent active (segments correct by construction).
    fn emit_combined_nodes(
        &self,
        nodes: &[&Node],
        output: &mut Output,
        indent: usize,
        trailing_comment: Option<&CommentNode>,
    ) {
        let has_expressions = nodes.iter().any(|node| self.node_has_expressions(node));

        // ── Phase 1: Analyze ──
        // Emit to a temp buffer to get the raw content string.
        // Segments from this pass are discarded.
        let mut temp = Output::new();
        for node in nodes {
            self.emit_node_content(node, &mut temp, has_expressions);
        }
        let (content, _) = temp.finish();
        let info = analyze_combined_content(&content);

        // If content is empty after trimming, just emit blank lines.
        // The first newline is structural (line break between parent and child),
        // only additional newlines are intentional blank lines.
        if info.is_empty {
            let newline_count = content
                .chars()
                .filter(|&c| c == '\n')
                .count()
                .saturating_sub(1);
            for _ in 0..newline_count {
                output.newline();
            }
            return;
        }

        // ── Phase 2: Emit ──
        // Leading blank lines
        for _ in 0..info.leading_newlines {
            output.newline();
        }

        self.indent(output, indent);

        // Yield prefix
        if has_expressions {
            if info.is_multiline {
                output.push("yield f\"\"\"\\");
                output.newline();
            } else {
                output.push("yield f\"\"\"");
            }
        } else if info.is_multiline {
            output.push("yield \"\"\"\\");
            output.newline();
        } else {
            output.push("yield \"\"\"");
        }

        // Emit content with formatting-aware Output:
        //   skip_next  → discard leading whitespace
        //   begin_dedent → strip anchor-indent spaces at each content line start
        // Segments recorded by emit_node_content are correct because
        // Output.position() reflects the actual post-skip/dedent output.
        output.skip_next(info.leading_skip);
        if info.anchor_indent > 0 {
            output.begin_dedent(info.anchor_indent);
        }

        for node in nodes {
            self.emit_node_content(node, output, has_expressions);
        }

        if info.anchor_indent > 0 {
            output.end_dedent();
        }

        // Clean up trailing whitespace.
        // For multiline content that naturally ends with \n (from anchor-dedented
        // trailing indentation), preserve the newline so """ goes on its own line.
        // Otherwise trim everything so """ stays on the content line.
        if info.is_multiline && info.has_trailing_newline {
            output.trim_trailing_spaces();
        } else {
            output.trim_trailing();
        }

        // Yield suffix
        output.push("\"\"\"");
        if let Some(comment) = trailing_comment {
            output.push("  ");
            output.push(&comment.text);
        }
        output.newline();

        // Preserved trailing blank lines
        for _ in 0..info.trailing_blank_lines {
            output.newline();
        }
    }

    /// Emit the content of a node as part of a string literal
    fn emit_node_content(&self, node: &Node, output: &mut Output, in_fstring: bool) {
        match node {
            Node::Text(text) => {
                if in_fstring {
                    // Escape braces so they're literal in the f-string
                    output.push(&text.content.replace('{', "{{").replace('}', "}}"));
                } else {
                    output.push(&text.content);
                }
            }
            Node::Expression(expr) if in_fstring => {
                if expr.expr.contains("safe(") {
                    output.use_helper("safe");
                }
                if let Some(lowered) = lower_interpolation(expr) {
                    output.push("{");
                    print_expr(output, &lowered);
                    output.push("}");
                    return;
                }

                let has_format_extras =
                    expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
                let (start, end) = if has_format_extras {
                    // Format spec, conversion, or debug: emit raw (no escape wrapper)
                    output.push("{");
                    let start = output.position();
                    output.push(&expr.expr);
                    if expr.debug {
                        output.push("=");
                    }
                    if let Some(conv) = expr.conversion {
                        output.push("!");
                        output.push(&conv.to_string());
                    }
                    if let Some(ref spec) = expr.format_spec {
                        output.push(":");
                        output.push(spec);
                    }
                    let end = output.position();
                    output.push("}");
                    (start, end)
                } else {
                    let start = output.position();
                    output.push("{");
                    output.push(&expr.expr);
                    output.push("}");
                    let end = output.position();
                    (start, end)
                };

                // Source segment excludes braces, just the inner expression
                let content_start = expr.range.start.byte + 1; // skip '{'
                let content_end = expr.range.end.byte - 1; // skip '}'

                output.add_segment(Segment {
                    language: Language::Python,
                    source_start: content_start,
                    source_end: content_end,
                    compiled_start: start,
                    compiled_end: end,
                    needs_injection: true,
                    html_prefix: None,
                });
            }
            Node::Element(el) => {
                self.emit_element_content(el, output, in_fstring);
            }
            _ => {}
        }
    }

    /// Emit element content as part of a string literal
    fn emit_element_content(&self, el: &ElementNode, output: &mut Output, in_fstring: bool) {
        output.push("<");
        output.push(&el.tag);

        // Emit attributes
        for attr in &el.attributes {
            self.emit_element_attribute(attr, output, in_fstring);
        }

        if el.self_closing {
            output.push(if html::is_void_element(&el.tag) {
                ">"
            } else {
                " />"
            });
        } else {
            output.push(">");

            // Emit children content
            for child in &el.children {
                self.emit_node_content(child, output, in_fstring);
            }

            output.push("</");
            output.push(&el.tag);
            output.push(">");
        }

        // Add HTML injection segments for this element's static HTML parts
        for seg in html_segments_for_element(el) {
            output.add_segment(seg);
        }
    }

    fn is_boolean_attribute(&self, name: &str) -> bool {
        crate::html::is_boolean_attribute(name)
    }

    /// Emit attribute content as part of a string literal
    fn emit_element_attribute(&self, attr: &Attribute, output: &mut Output, in_fstring: bool) {
        // Non-dynamic arms each emit and return. Dynamic arms build a
        // (scaffold, helper call) pair that the shared block below emits.
        let (scaffold, expr) = match &attr.kind {
            AttributeKind::Static { name, value } => {
                output.push(" ");
                output.push(name);
                output.push("=\"");
                output.push(&escape_html_attr_quotes(value));
                output.push("\"");
                return;
            }
            AttributeKind::Boolean { name } => {
                output.push(" ");
                output.push(name);
                return;
            }
            AttributeKind::SlotAssignment { .. } => return,
            AttributeKind::Template { name, value } => {
                if in_fstring {
                    output.push(" ");
                    output.push(name);
                    output.push("=\"");
                    // Emit template value with position tracking for each {expr}
                    // value_start_byte: skip past `name="` in the source
                    let value_start_byte = attr.range.start.byte + name.len() + 2;
                    let mut byte_offset = 0;
                    let mut chars = value.chars().peekable();
                    #[allow(clippy::while_let_on_iterator)]
                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            // Collect expression until closing }
                            let expr_byte_start = byte_offset + 1; // past '{'
                            byte_offset += ch.len_utf8();
                            let mut expr = String::new();
                            let mut depth = 1;
                            while let Some(inner) = chars.next() {
                                byte_offset += inner.len_utf8();
                                if inner == '{' {
                                    depth += 1;
                                    expr.push(inner);
                                } else if inner == '}' {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                    expr.push(inner);
                                } else {
                                    expr.push(inner);
                                }
                            }
                            let expr_byte_end = byte_offset - 1; // before '}'
                            // Template value is parsed here, so rename the extracted expr.
                            let safe_expr = rename_reserved_keywords(expr.trim());
                            output.push("{");
                            let code = code_span(
                                safe_expr,
                                value_start_byte + expr_byte_start,
                                value_start_byte + expr_byte_end,
                            );
                            print_expr(output, &helper_call("escape", code));
                            output.push("}");
                        } else if ch == '"' {
                            output.push("&quot;");
                            byte_offset += ch.len_utf8();
                        } else {
                            output.push(&ch.to_string());
                            byte_offset += ch.len_utf8();
                        }
                    }
                    output.push("\"");
                }
                return;
            }

            AttributeKind::Expression {
                name,
                expr,
                expr_range,
            } => {
                if !in_fstring {
                    return;
                }
                // Already renamed in the AST by ReservedKeywordPlugin.
                let safe_expr = expr.trim().to_string();
                // expr_range includes {expr}, skip braces for injection.
                let content_start = expr_range.start.byte + 1;
                let content_end = expr_range.end.byte - 1;
                let code = code_span(safe_expr, content_start, content_end);
                match name.as_str() {
                    "class" => (Scaffold::Value(name), helper_call("render_class", code)),
                    "style" => (Scaffold::Value(name), helper_call("render_style", code)),
                    n if self.is_boolean_attribute(n) => {
                        (Scaffold::Whole, render_attr_call(name, code))
                    }
                    _ => (Scaffold::Value(name), helper_call("escape", code)),
                }
            }

            AttributeKind::Shorthand { name, expr_range } => {
                if !in_fstring {
                    return;
                }
                // Shorthand maps one AST field to two outputs: HTML attr name
                // stays, Python value variable renames. Rename here.
                let var_name = rename_reserved_keywords(name);
                // Shorthand expr_range.end points TO closing brace (not past it),
                // so content_end = end.byte gives exclusive end of name content.
                let content_start = expr_range.start.byte + 1;
                let content_end = expr_range.end.byte;
                let code = code_span(var_name, content_start, content_end);
                match name.as_str() {
                    "class" => (Scaffold::Value(name), helper_call("render_class", code)),
                    "style" => (Scaffold::Value(name), helper_call("render_style", code)),
                    "data" => (Scaffold::Whole, helper_call("render_data", code)),
                    "aria" => (Scaffold::Whole, helper_call("render_aria", code)),
                    _ => (Scaffold::Whole, render_attr_call(name, code)),
                }
            }

            AttributeKind::Spread { expr, expr_range } => {
                if !in_fstring {
                    return;
                }
                // Already renamed in the AST by ReservedKeywordPlugin.
                let safe_expr = expr.trim().to_string();
                // Spread expr_range is {**expr}; skip 3 chars for "{**".
                let content_start = expr_range.start.byte + 3;
                let content_end = expr_range.end.byte;
                let code = code_span(safe_expr, content_start, content_end);
                (Scaffold::Whole, helper_call("spread_attrs", code))
            }
        };

        // Shared scaffold emission for dynamic arms.
        match scaffold {
            Scaffold::Value(name) => {
                output.push(" ");
                output.push(name);
                output.push("=\"{");
                print_expr(output, &expr);
                output.push("}\"");
            }
            Scaffold::Whole => {
                output.push("{");
                print_expr(output, &expr);
                output.push("}");
            }
        }
    }

    /// Convert {expr} in template string to {escape(expr)} for f-string output.
    /// Also escapes double quotes in static parts as &quot; for valid HTML attributes.
    fn convert_template_expressions(&self, template: &str) -> String {
        let mut result = String::new();
        let mut chars = template.chars().peekable();

        #[allow(clippy::while_let_on_iterator)]
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Found start of expression, collect until closing }
                let mut expr = String::new();
                let mut depth = 1;
                while let Some(inner) = chars.next() {
                    if inner == '{' {
                        depth += 1;
                        expr.push(inner);
                    } else if inner == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr.push(inner);
                    } else {
                        expr.push(inner);
                    }
                }
                // Emit as direct escape() call
                result.push_str("{escape(");
                result.push_str(&expr);
                result.push_str(")}");
            } else if ch == '"' {
                result.push_str("&quot;");
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn emit_node(&self, node: &Node, output: &mut Output, indent: usize) {
        match node {
            Node::Text(text) => self.emit_text(text, output, indent),
            Node::Expression(expr) => self.emit_expression(expr, output, indent),
            Node::Comment(comment) => self.emit_comment(comment, output, indent),
            Node::Element(el) => self.emit_element(el, output, indent),
            Node::Component(c) => self.emit_component(c, output, indent),
            Node::Fragment(f) => self.emit_fragment(f, output, indent),
            Node::Slot(s) => self.emit_slot(s, output, indent),
            Node::If(if_node) => self.emit_if(if_node, output, indent),
            Node::For(for_node) => self.emit_for(for_node, output, indent),
            Node::Match(match_node) => self.emit_match(match_node, output, indent),
            Node::While(while_node) => self.emit_while(while_node, output, indent),
            Node::With(with_node) => self.emit_with(with_node, output, indent),
            Node::Try(try_node) => self.emit_try(try_node, output, indent),
            Node::Statement(stmt) => self.emit_statement(stmt, output, indent),
            Node::Definition(def) => self.emit_definition(def, output, indent),
            Node::Import(import) => self.emit_import(import, output, indent),
            Node::Parameter(_) => {} // Parameters handled separately
            Node::Decorator(dec) => self.emit_decorator(dec, output, indent),
        }
    }

    fn emit_text(&self, text: &TextNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("yield \"");
        output.push(&escape_string(&text.content));
        output.push("\"");
        output.newline();
    }

    fn emit_comment(&self, comment: &CommentNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push(&comment.text);
        output.newline();
    }

    fn emit_expression(&self, expr: &ExpressionNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        if expr.expr.contains("safe(") {
            output.use_helper("safe");
        }
        if let Some(lowered) = lower_interpolation(expr) {
            output.push("yield ");
            print_expr(output, &lowered);
            output.newline();
            return;
        }

        let has_format_extras =
            expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
        if has_format_extras {
            // Format spec, conversion, or debug: emit as f-string
            output.push("yield f\"{");
            let start = output.position();
            output.push(&expr.expr);
            let end = output.position();
            if expr.debug {
                output.push("=");
            }
            if let Some(conv) = expr.conversion {
                output.push("!");
                output.push(&conv.to_string());
            }
            if let Some(ref spec) = expr.format_spec {
                output.push(":");
                output.push(spec);
            }
            output.push("}\"");
            // Source range excludes braces: range.start + 1 to range.end - 1
            output.add_segment(Segment {
                language: Language::Python,
                source_start: expr.range.start.byte + 1,
                source_end: expr.range.end.byte - 1,
                compiled_start: start,
                compiled_end: end,
                needs_injection: true,
                html_prefix: None,
            });
        } else {
            output.push("yield str(");
            let start = output.position();
            output.push(&expr.expr);
            let end = output.position();
            output.push(")");
            output.add_segment(Segment {
                language: Language::Python,
                source_start: expr.range.start.byte + 1,
                source_end: expr.range.end.byte - 1,
                compiled_start: start,
                compiled_end: end,
                needs_injection: true,
                html_prefix: None,
            });
        }
        output.newline();
    }

    fn emit_element(&self, el: &ElementNode, output: &mut Output, indent: usize) {
        // Check if any attribute requires f-string interpolation
        let needs_fstring = el.attributes.iter().any(|attr| {
            matches!(
                attr.kind,
                AttributeKind::Expression { .. }
                    | AttributeKind::Template { .. }
                    | AttributeKind::Shorthand { .. }
                    | AttributeKind::Spread { .. }
            )
        });

        self.indent(output, indent);
        if needs_fstring {
            output.push("yield f\"\"\"<");
        } else {
            output.push("yield \"\"\"<");
        }
        output.push(&el.tag);

        // Emit attributes
        for attr in &el.attributes {
            self.emit_element_attribute(attr, output, needs_fstring);
        }

        if el.self_closing {
            output.push(if html::is_void_element(&el.tag) {
                ">\"\"\""
            } else {
                " />\"\"\""
            });
            output.newline();
        } else {
            output.push(">\"\"\"");
            output.newline();

            // Emit children using emit_nodes for proper grouping
            let refs: Vec<&Node> = el.children.iter().collect();
            self.emit_nodes(&refs, output, indent);

            // Closing tag
            self.indent(output, indent);
            output.push("yield \"\"\"</");
            output.push(&el.tag);
            output.push(">\"\"\"");
            output.newline();
        }

        // Add HTML injection segments for this element
        for seg in html_segments_for_element(el) {
            output.add_segment(seg);
        }
    }

    /// Generate a local function name for one component call slot.
    fn component_to_func_name(&self, component: &str, slot: Option<&str>) -> String {
        let mut result = String::from("_");
        let mut prev_was_separator = false;
        for (i, ch) in component.chars().enumerate() {
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
        result.push('_');
        result.push_str(slot.unwrap_or(DEFAULT_SLOT_PARAM));
        result
    }

    fn emit_component(&self, c: &ComponentNode, output: &mut Output, indent: usize) {
        let has_content = !c.children.is_empty();
        let mut named_slots: Vec<_> = c.slots.iter().collect();
        named_slots.sort_by_key(|(name, _)| *name);
        let has_body = has_content || !named_slots.is_empty();

        if has_body {
            self.indent(output, indent);
            output.push("# <{");
            output.push(&c.name);
            output.push("}>");
            output.newline();
        }

        if has_content {
            let func_name = self.component_to_func_name(&c.name, None);
            self.indent(output, indent);
            output.push("def ");
            output.push(&func_name);
            output.push("():");
            output.newline();
            self.emit_body_or_pass(&c.children, output, indent + 1);
        }

        for (name, body) in &named_slots {
            let func_name = self.component_to_func_name(&c.name, Some(name));
            self.indent(output, indent);
            output.push("def ");
            output.push(&func_name);
            output.push("():");
            output.newline();
            self.emit_body_or_pass(body, output, indent + 1);
        }

        self.indent(output, indent);
        output.push("yield from ");
        let name_compiled_start = output.position();
        output.push(&c.name);
        let name_compiled_end = output.position();
        output.push(".stream(");

        let mut first = true;
        if has_content {
            output.push(DEFAULT_SLOT_PARAM);
            output.push("=");
            output.push(&self.component_to_func_name(&c.name, None));
            output.push("()");
            first = false;
        }
        for (name, _) in &named_slots {
            if !first {
                output.push(", ");
            }
            output.push(name);
            output.push("=");
            output.push(&self.component_to_func_name(&c.name, Some(name)));
            output.push("()");
            first = false;
        }
        for attr in &c.attributes {
            if matches!(attr.kind, AttributeKind::SlotAssignment { .. }) {
                continue;
            }
            if !first {
                output.push(", ");
            }
            self.emit_component_attribute(attr, output);
            first = false;
        }
        output.push(")");
        output.newline();

        if has_body {
            self.indent(output, indent);
            output.push("# </{");
            output.push(&c.name);
            output.push("}>");
            output.newline();
        }

        // Add Python segment for the component name in the opening tag
        // This enables go-to-definition and highlighting for the name
        output.add_segment(Segment {
            language: Language::Python,
            source_start: c.name_range.start.byte,
            source_end: c.name_range.end.byte,
            compiled_start: name_compiled_start,
            compiled_end: name_compiled_end,
            needs_injection: true,
            html_prefix: None,
        });

        // Add Python segment for the component name in the closing tag.
        // needs_injection: false — this is for highlighting only, not for
        // building the virtual Python file (which would duplicate the name).
        if let Some(ref cs) = c.close_range {
            // Closing tag is </{Name}> — name starts at byte+3 (skip "</{"), ends at byte-2 (skip "}>")
            let close_name_start = cs.start.byte + 3;
            let close_name_end = cs.end.byte - 2;
            if close_name_end > close_name_start {
                output.add_segment(Segment {
                    language: Language::Python,
                    source_start: close_name_start,
                    source_end: close_name_end,
                    compiled_start: 0,
                    compiled_end: 0,
                    needs_injection: false,
                    html_prefix: None,
                });
            }
        }

        // Add HTML segments for component tag angle brackets,
        // splitting around attribute expression spans to avoid overlap
        let brace_open = c.name_range.start.byte - 1;
        let brace_close = c.name_range.end.byte;
        let attr_expr_spans = collect_component_attr_expr_spans(&c.attributes);
        for seg in html_segments_for_component(
            &c.range,
            c.close_range.as_ref(),
            brace_open,
            brace_close,
            &attr_expr_spans,
        ) {
            output.add_segment(seg);
        }
    }

    /// Emit a single attribute as a Python keyword argument in a component call
    fn emit_component_attribute(&self, attr: &Attribute, output: &mut Output) {
        match &attr.kind {
            AttributeKind::Static { name, value } => {
                output.push(name);
                output.push("=\"");
                output.push(&escape_string(value));
                output.push("\"");
            }
            AttributeKind::Expression {
                name,
                expr,
                expr_range,
            } => {
                let content_start = expr_range.start.byte + 1;
                let content_end = expr_range.end.byte - 1;
                output.push(name);
                output.push("=");
                let s = output.position();
                output.push(expr);
                let e = output.position();
                output.add_segment(Segment {
                    language: Language::Python,
                    source_start: content_start,
                    source_end: content_end,
                    compiled_start: s,
                    compiled_end: e,
                    needs_injection: true,
                    html_prefix: None,
                });
            }
            AttributeKind::Boolean { name } => {
                output.push(name);
                output.push("=True");
            }
            AttributeKind::Shorthand { name, expr_range } => {
                // name is already renamed in the AST by ReservedKeywordPlugin.
                let content_start = expr_range.start.byte + 1;
                let content_end = expr_range.end.byte;
                output.push(name);
                output.push("=");
                let s = output.position();
                output.push(name);
                let e = output.position();
                output.add_segment(Segment {
                    language: Language::Python,
                    source_start: content_start,
                    source_end: content_end,
                    compiled_start: s,
                    compiled_end: e,
                    needs_injection: true,
                    html_prefix: None,
                });
            }
            AttributeKind::Spread { expr, expr_range } => {
                let content_start = expr_range.start.byte + 3;
                let content_end = expr_range.end.byte;
                output.push("**");
                let s = output.position();
                output.push(expr.trim());
                let e = output.position();
                output.add_segment(Segment {
                    language: Language::Python,
                    source_start: content_start,
                    source_end: content_end,
                    compiled_start: s,
                    compiled_end: e,
                    needs_injection: true,
                    html_prefix: None,
                });
            }
            AttributeKind::Template { name, value } => {
                output.push(name);
                output.push("=f\"");
                let converted = self.convert_template_expressions(value);
                if converted.contains("escape(") {
                    output.use_helper("escape");
                }
                output.push(&converted);
                output.push("\"");
            }
            AttributeKind::SlotAssignment { .. } => {
                // Slot assignments are handled separately by the slot mechanism
            }
        }
    }

    fn emit_fragment(&self, f: &FragmentNode, output: &mut Output, indent: usize) {
        let refs: Vec<&Node> = f.children.iter().collect();
        self.emit_nodes(&refs, output, indent);
    }

    fn emit_slot(&self, s: &SlotNode, output: &mut Output, indent: usize) {
        if s.is_fill {
            let refs: Vec<&Node> = s.fallback.iter().collect();
            self.emit_nodes(&refs, output, indent);
            if s.close_range.is_some() {
                let brace_open = s.range.start.byte + 1;
                let brace_close = s.range.end.byte - 2;
                for seg in html_segments_for_component(
                    &s.range,
                    s.close_range.as_ref(),
                    brace_open,
                    brace_close,
                    &[],
                ) {
                    output.add_segment(seg);
                }
            }
            return;
        }

        // Emit conditional yield from for slot content
        let slot_var = slot_param_name(s.name.as_deref());

        // Slot label for comments: {...} for default, {...name} for named
        let slot_label = if let Some(name) = &s.name {
            format!("{{...{}}}", name)
        } else {
            "{...}".to_string()
        };

        // Opening comment
        self.indent(output, indent);
        output.push("# <");
        output.push(&slot_label);
        output.push(">");
        output.newline();

        self.indent(output, indent);
        output.push("if ");
        output.push(&slot_var);
        output.push(" is not None:");
        output.newline();

        self.indent(output, indent + 1);
        output.push("yield from ");
        output.push(&slot_var);
        output.newline();

        if !s.fallback.is_empty() {
            self.indent(output, indent);
            output.push("else:");
            output.newline();

            let refs: Vec<&Node> = s.fallback.iter().collect();
            self.emit_nodes(&refs, output, indent + 1);
        }

        // Closing comment
        self.indent(output, indent);
        output.push("# </");
        output.push(&slot_label);
        output.push(">");
        output.newline();

        // Add HTML segments for tag-form slot angle brackets (<{...name}> / </{...name}>)
        // Slots have no attributes, so no expression spans to exclude
        if s.close_range.is_some() {
            let brace_open = s.range.start.byte + 1;
            let brace_close = s.range.end.byte - 2;
            for seg in html_segments_for_component(
                &s.range,
                s.close_range.as_ref(),
                brace_open,
                brace_close,
                &[],
            ) {
                output.add_segment(seg);
            }
        }
    }

    fn emit_if(&self, if_node: &IfNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("if ");
        // Strip trailing `:` so it does not land in the injection segment
        let condition = if_node.condition.trim_end_matches(':').trim();
        print_code(output, &code_from(condition, if_node.condition_range));
        output.push(":");
        output.newline();

        self.emit_body_or_pass(&if_node.then_branch, output, indent + 1);

        for (condition, condition_range, body) in &if_node.elif_branches {
            self.indent(output, indent);
            output.push("elif ");
            let condition = condition.trim_end_matches(':').trim();
            print_code(output, &code_from(condition, *condition_range));
            output.push(":");
            output.newline();

            self.emit_body_or_pass(body, output, indent + 1);
        }

        if let Some(else_branch) = &if_node.else_branch {
            self.indent(output, indent);
            output.push("else:");
            output.newline();

            self.emit_body_or_pass(else_branch, output, indent + 1);
        }
    }

    fn emit_for(&self, for_node: &ForNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        if for_node.is_async {
            output.push("async for ");
        } else {
            output.push("for ");
        }
        let iterable = for_node.iterable.trim_end_matches(':').trim();
        let source = format!("{} in {}", for_node.binding, iterable);
        let end_byte = for_node.iterable_range.start.byte + iterable.len();
        print_code(
            output,
            &Code {
                source,
                range: TextRange {
                    start: for_node.binding_range.start,
                    end: Position {
                        byte: end_byte,
                        line: 0,
                        col: 0,
                    },
                },
            },
        );
        output.push(":");
        output.newline();

        self.emit_body_or_pass(&for_node.body, output, indent + 1);
    }

    fn emit_match(&self, match_node: &MatchNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("match ");
        let expr = match_node.expr.trim_end_matches(':').trim();
        print_code(output, &code_from(expr, match_node.expr_range));
        output.push(":");
        output.newline();

        for case in match_node.cases.iter() {
            self.indent(output, indent + 1);
            output.push("case ");
            let pattern = case.pattern.trim_end_matches(':').trim();
            print_code(output, &code_from(pattern, case.pattern_range));
            output.push(":");
            output.newline();

            self.emit_body_or_pass(&case.body, output, indent + 2);
        }
    }

    fn emit_while(&self, while_node: &WhileNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("while ");
        let condition = while_node.condition.trim_end_matches(':').trim();
        print_code(output, &code_from(condition, while_node.condition_range));
        output.push(":");
        output.newline();

        self.emit_body_or_pass(&while_node.body, output, indent + 1);
    }

    fn emit_with(&self, with_node: &WithNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        if with_node.is_async {
            output.push("async with ");
        } else {
            output.push("with ");
        }
        let items = with_node.items.trim_end_matches(':').trim();
        print_code(output, &code_from(items, with_node.items_range));
        output.push(":");
        output.newline();

        self.emit_body_or_pass(&with_node.body, output, indent + 1);
    }

    fn emit_try(&self, try_node: &TryNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("try:");
        output.newline();

        self.emit_body_or_pass(&try_node.body, output, indent + 1);

        for except in &try_node.except_clauses {
            self.indent(output, indent);
            output.push("except");
            if let Some(exception) = &except.exception {
                output.push(" ");
                let exception = exception.trim_end_matches(':').trim();
                let range = except.exception_range.unwrap_or(TextRange::synthetic());
                print_code(output, &code_from(exception, range));
            }
            output.push(":");
            output.newline();

            self.emit_body_or_pass(&except.body, output, indent + 1);
        }

        if let Some(else_clause) = &try_node.else_clause {
            self.indent(output, indent);
            output.push("else:");
            output.newline();

            self.emit_body_or_pass(else_clause, output, indent + 1);
        }

        if let Some(finally_clause) = &try_node.finally_clause {
            self.indent(output, indent);
            output.push("finally:");
            output.newline();

            self.emit_body_or_pass(finally_clause, output, indent + 1);
        }
    }

    fn emit_statement(&self, stmt: &StatementNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        // Re-indent continuation lines for multiline statements
        let source = if stmt.stmt.contains('\n') {
            let continuation_indent = "    ".repeat(indent);
            stmt.stmt.replace('\n', &format!("\n{continuation_indent}"))
        } else {
            stmt.stmt.clone()
        };
        print_code(
            output,
            &Code {
                source,
                range: stmt.range,
            },
        );
        output.newline();
    }

    fn emit_definition(&self, def: &DefinitionNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        print_code(
            output,
            &Code {
                source: def.signature.clone(),
                range: def.signature_range,
            },
        );
        output.newline();

        self.emit_body_or_pass(&def.body, output, indent + 1);
    }

    fn emit_import(&self, import: &ImportNode, output: &mut Output, _indent: usize) {
        print_code(
            output,
            &Code {
                source: import.stmt.clone(),
                range: import.range,
            },
        );
        output.newline();
    }

    fn emit_decorator(&self, dec: &DecoratorNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        print_code(
            output,
            &Code {
                source: dec.decorator.clone(),
                range: dec.range,
            },
        );
        output.newline();
    }

    fn indent(&self, output: &mut Output, level: usize) {
        for _ in 0..level {
            output.push("    ");
        }
    }

    /// Emit one signature parameter (`name: type = default,`). User params map
    /// back to source for injection; synthetic (slot) params do not.
    fn emit_signature_param(&self, param: &ParameterNode, output: &mut Output, sig_indent: &str) {
        output.push(sig_indent);
        let start = output.position();
        output.push(&param.name);
        if let Some(type_hint) = &param.type_hint {
            output.push(": ");
            output.push(type_hint);
        }
        if let Some(default) = &param.default {
            output.push(" = ");
            output.push(default);
        }
        let end = output.position();

        if !param.range.is_synthetic() {
            output.add_segment(Segment {
                language: Language::Python,
                source_start: param.range.start.byte,
                source_end: param.range.end.byte,
                compiled_start: start,
                compiled_end: end,
                needs_injection: true,
                html_prefix: None,
            });
        }
        output.push(",");
        output.newline();
    }

    fn function_parameters<'a>(&self, function: &'a Function) -> Vec<&'a ParameterNode> {
        function
            .params
            .iter()
            .filter_map(|node| match node {
                Node::Parameter(param) => Some(param),
                _ => None,
            })
            .collect()
    }

    fn emit_render_function(
        &self,
        name: &str,
        name_range: Option<TextRange>,
        function: &Function,
        output: &mut Output,
    ) {
        for decorator in &function.decorators {
            self.emit_decorator(decorator, output, 0);
        }

        if function.is_async {
            output.push("async def ");
        } else {
            output.push("def ");
        }
        let name_start = output.position();
        output.push(name);
        if let Some(range) = name_range {
            output.add_segment(Segment {
                language: Language::Python,
                source_start: range.start.byte,
                source_end: range.end.byte,
                compiled_start: name_start,
                compiled_end: output.position(),
                needs_injection: true,
                html_prefix: None,
            });
        }

        let parameters = self.function_parameters(function);
        let positional: Vec<&ParameterNode> = parameters
            .iter()
            .copied()
            .filter(|param| param.kind == ParamKind::Positional)
            .collect();
        let keyword_only: Vec<&ParameterNode> = parameters
            .iter()
            .copied()
            .filter(|param| param.kind == ParamKind::KeywordOnly)
            .collect();
        let var_keyword = parameters
            .iter()
            .copied()
            .find(|param| param.kind == ParamKind::VarKeyword);

        if positional.is_empty() && keyword_only.is_empty() && var_keyword.is_none() {
            output.push("():");
            output.newline();
        } else {
            output.push("(");
            output.newline();
            let indent = "        ";

            for param in positional {
                self.emit_signature_param(param, output, indent);
            }
            if !keyword_only.is_empty() {
                output.push(indent);
                output.push("*,");
                output.newline();
            }
            for param in keyword_only {
                self.emit_signature_param(param, output, indent);
            }
            if let Some(param) = var_keyword {
                output.push(indent);
                output.push(&param.name);
                if let Some(type_hint) = &param.type_hint {
                    output.push(": ");
                    output.push(type_hint);
                }
                output.push(",");
                output.newline();
            }

            output.push("):");
            output.newline();
        }

        let body: Vec<&Node> = function.body.iter().collect();
        if body.is_empty() || self.is_effectively_empty(&body) {
            self.indent(output, 1);
            output.push("yield from ()");
            output.newline();
        } else {
            self.emit_nodes(&body, output, 1);
        }
    }
}

impl Default for PythonGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for PythonGenerator {
    fn generate(&self, ast: &Ast, options: &CompileOptions) -> CompileResult {
        let mut output = Output::new();

        // Frontmatter and body are already split by the `lower` pass.
        let function = &ast.function;
        let parameters = self.function_parameters(function);
        let mut all_parameters = parameters.clone();
        for definition in &ast.definitions {
            all_parameters.extend(self.function_parameters(&definition.function));
        }
        let imports: Vec<&ImportNode> = function.imports.iter().collect();
        let header_comments: Vec<&CommentNode> = function.header_comments.iter().collect();

        // Emit user imports
        for import in &imports {
            let import_start = output.position();
            output.push(&import.stmt);
            let import_end = output.position();
            output.newline();

            output.add_segment(Segment {
                language: Language::Python,
                source_start: import.range.start.byte,
                source_end: import.range.end.byte,
                compiled_start: import_start,
                compiled_end: import_end,
                needs_injection: true,
                html_prefix: None,
            });
        }
        let runtime_import_offset = output.position();
        let function_name = options
            .function_name
            .as_deref()
            .map(to_pascal_case)
            .unwrap_or_else(|| "Render".to_string());

        if ast.mode == FileMode::Library {
            // Library statements define names used by component defaults and decorators.
            let body: Vec<&Node> = function.body.iter().collect();
            if !self.is_effectively_empty(&body) {
                self.emit_nodes(&body, &mut output, 0);
                output.newline();
                output.newline();
            }
        }

        for definition in &ast.definitions {
            self.emit_render_function(
                &definition.name,
                Some(definition.name_range),
                &definition.function,
                &mut output,
            );
            output.newline();
            output.newline();
        }

        if ast.mode == FileMode::ImplicitComponent {
            self.emit_render_function(&function_name, None, function, &mut output);
        }

        // Hyper runtime imports, in Helper::ALL order, for helpers actually emitted.
        let mut hyper_imports = Vec::new();
        if ast.mode == FileMode::ImplicitComponent || !ast.definitions.is_empty() {
            hyper_imports.push("component");
        }
        for helper in Helper::ALL {
            if output.helper_used(helper.import_name()) {
                hyper_imports.push(helper.import_name());
            }
        }

        let (mut code, tracked_segments) = output.finish();

        // Iterable import is needed when a param is typed with it (slot params).
        let needs_iterable = all_parameters.iter().any(|p| {
            p.type_hint
                .as_deref()
                .is_some_and(|t| t.contains("Iterable"))
        });

        // Detect typing constructs needed from parameter type hints
        let mut typing_imports: Vec<&str> = Vec::new();
        let all_type_hints: String = all_parameters
            .iter()
            .filter_map(|p| p.type_hint.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");

        if all_type_hints.contains("Any") {
            typing_imports.push("Any");
        }
        if all_type_hints.contains("Callable") {
            typing_imports.push("Callable");
        }
        if all_type_hints.contains("Optional") {
            typing_imports.push("Optional");
        }
        if all_type_hints.contains("Union") {
            typing_imports.push("Union");
        }
        if all_type_hints.contains("TypeVar") {
            typing_imports.push("TypeVar");
        }

        // Build import block
        let mut import_lines = String::new();

        // Add typing imports if needed
        if !typing_imports.is_empty() {
            import_lines.push_str(&print_import_from(&import_from("typing", &typing_imports)));
            import_lines.push('\n');
        }

        // Add Iterable import if needed
        if needs_iterable {
            import_lines.push_str(&print_import_from(&import_from(
                "collections.abc",
                &["Iterable"],
            )));
            import_lines.push('\n');
        }

        // Add Hyper runtime imports
        if !hyper_imports.is_empty() {
            import_lines.push_str(&print_import_from(&import_from(
                "hyperhtml",
                &hyper_imports,
            )));
            import_lines.push('\n');
        }
        import_lines.push_str("\n\n"); // Two blank lines before function (PEP 8)

        // Add header comments (above --- separator)
        for comment in &header_comments {
            import_lines.push_str(&comment.text);
            import_lines.push('\n');
        }

        let import_offset = import_lines.len();
        code.insert_str(runtime_import_offset, &import_lines);

        // Adjust segments and collect IDE metadata when ranges are requested.
        let (segments, expression_braces) = if options.include_ranges {
            // Adjust tracked segments by the import line offset, but only for segments
            // at or after the insertion point (user imports come before it)
            let segments: Vec<crate::generate::Segment> = tracked_segments
                .into_iter()
                .map(|mut s| {
                    if s.compiled_start >= runtime_import_offset {
                        s.compiled_start += import_offset;
                        s.compiled_end += import_offset;
                    }
                    s
                })
                .collect();

            // Collect expression brace positions from the AST
            let byte_braces = collect_expression_braces(ast);
            let expression_braces = convert_braces_to_utf16(&ast.source, &byte_braces);

            (segments, expression_braces)
        } else {
            (Vec::new(), Vec::new())
        };

        CompileResult {
            code,
            file_mode: ast.mode,
            component_name: (ast.mode == FileMode::ImplicitComponent).then_some(function_name),
            segments,
            expression_braces,
        }
    }
}

/// Formatting parameters for a combined-content yield block.
struct CombinedContentInfo {
    /// Characters to skip from the start (leading whitespace).
    leading_skip: usize,
    /// Blank lines to emit before the yield statement.
    leading_newlines: usize,
    /// Spaces to strip at each subsequent content-line start.
    anchor_indent: usize,
    /// Whether the content spans multiple lines after processing.
    is_multiline: bool,
    /// Whether the processed content ends with `\n` (determines `"""` placement).
    /// True when the last line of trimmed content is all spaces that get fully
    /// stripped by anchor dedent, leaving an empty line preceded by `\n`.
    has_trailing_newline: bool,
    /// Blank lines to emit after the yield statement.
    trailing_blank_lines: usize,
    /// Content is empty after trimming — emit blank lines only.
    is_empty: bool,
}

/// Analyze raw combined content to determine formatting parameters.
///
/// The content string is the concatenation of all node outputs (text, expressions,
/// element tags) before any formatting. This function figures out how much leading
/// whitespace to skip, how much to dedent subsequent lines, and whether the result
/// is multiline.
fn analyze_combined_content(content: &str) -> CombinedContentInfo {
    // Trim leading whitespace (newlines + spaces)
    let trimmed = content.trim_start_matches(['\n', ' ']);
    let leading_trimmed = content.len() - trimmed.len();

    // Strip trailing newlines: one is the line ending, extras are blank lines to preserve
    let trimmed = trimmed.strip_suffix('\n').unwrap_or(trimmed);
    let trailing_blank_lines = trimmed.chars().rev().take_while(|&c| c == '\n').count();
    let trimmed = trimmed.trim_end_matches('\n');

    // Count leading blank lines. The first newline is always structural (the
    // line break between a parent tag/statement and the first child), so only
    // additional newlines represent intentional blank lines.
    let leading_newlines = content[..leading_trimmed]
        .chars()
        .filter(|&c| c == '\n')
        .count()
        .saturating_sub(1);

    if trimmed.is_empty() {
        return CombinedContentInfo {
            leading_skip: 0,
            leading_newlines: 0,
            anchor_indent: 0,
            is_multiline: false,
            has_trailing_newline: false,
            trailing_blank_lines: 0,
            is_empty: true,
        };
    }

    // Anchor indent: the indentation of the first content line.
    // This is the number of spaces between the last newline in the leading
    // whitespace and the start of content.
    let last_newline_pos = content[..leading_trimmed].rfind('\n');
    let anchor_indent = match last_newline_pos {
        Some(pos) => leading_trimmed - (pos + 1),
        None => leading_trimmed,
    };

    // Multiline check: after stripping trailing whitespace, does content contain \n?
    // Anchor dedent doesn't add or remove newlines, so we can check `trimmed` directly.
    let is_multiline = trimmed.trim_end_matches([' ', '\t', '\n']).contains('\n');

    // Does the processed content end with \n?
    // This happens when the last line of `trimmed` consists entirely of spaces
    // that get fully stripped by anchor dedent, leaving an empty line.
    // Example: `<div>\n        </div>\n    ` with anchor=8 → last line `    `
    // becomes empty after dedent, so processed content ends with `\n`.
    let has_trailing_newline = if anchor_indent > 0 && trimmed.contains('\n') {
        let last_line = trimmed.rsplit('\n').next().unwrap_or("");
        if last_line.is_empty() {
            false
        } else {
            // After dedent, does the last line become empty?
            let stripped = if last_line.len() >= anchor_indent
                && last_line[..anchor_indent].chars().all(|c| c == ' ')
            {
                &last_line[anchor_indent..]
            } else {
                last_line.trim_start_matches(' ')
            };
            stripped.is_empty()
        }
    } else {
        false
    };

    // leading_skip counts characters (not bytes), matching Output::skip_next's semantics.
    // Leading content is always ASCII whitespace, so chars == bytes.
    let leading_skip = content[..leading_trimmed].chars().count();

    CombinedContentInfo {
        leading_skip,
        leading_newlines,
        anchor_indent,
        is_multiline,
        has_trailing_newline,
        trailing_blank_lines,
        is_empty: false,
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Escape double quotes as &quot; for HTML attribute values.
/// This is needed when single-quoted source values contain double quotes.
fn escape_html_attr_quotes(s: &str) -> String {
    s.replace('"', "&quot;")
}

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

fn import_from(module: &str, names: &[&str]) -> StmtImportFrom {
    StmtImportFrom {
        module: Some(Identifier::new(module)),
        names: names
            .iter()
            .map(|n| Alias {
                name: Identifier::new(*n),
                asname: None,
            })
            .collect(),
        level: 0,
    }
}

/// Build a `Code` whose source spans `[start, start + source.len())` in `.hyper`.
/// Used for control-flow conditions where trimming `:` only shrinks the tail.
/// Synthetic input range stays synthetic so the printer skips injection.
fn code_from(source: &str, range: TextRange) -> Code {
    let adjusted = if range.is_synthetic() {
        range
    } else {
        TextRange {
            start: range.start,
            end: Position {
                byte: range.start.byte + source.len(),
                line: 0,
                col: 0,
            },
        }
    };
    Code {
        source: source.to_string(),
        range: adjusted,
    }
}
