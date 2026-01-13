use super::{GenerateOptions, GenerateResult, Generator, Output, Range, RangeType};
use crate::ast::*;

pub struct PythonGenerator;

impl PythonGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Convert reserved Python keywords to safe variable names
    fn safe_var_name(&self, name: &str) -> String {
        match name {
            "class" => "_class".to_string(),
            "type" => "_type".to_string(),
            _ => name.to_string(),
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

                // Emit combined nodes as a single string/f-string
                self.emit_combined_nodes(&nodes[i..j], output, indent);
                i = j;
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
                // Check if element has dynamic attributes (including spreads) or expression children
                el.attributes.iter().any(|attr| {
                    !matches!(attr.kind, AttributeKind::Static { .. } | AttributeKind::Boolean { .. })
                })
                    || el.children.iter().any(|child| self.node_has_expressions(child))
            }
            _ => false,
        }
    }

    /// Emit consecutive text/expression/element nodes as a single string literal
    fn emit_combined_nodes(&self, nodes: &[&Node], output: &mut Output, indent: usize) {
        self.indent(output, indent);

        // Check if any node contains expressions (recursively)
        let has_expressions = nodes.iter().any(|node| self.node_has_expressions(node));

        output.push("_parts.append(");
        if has_expressions {
            output.push("f\"\"\"");
        } else {
            output.push("\"\"\"");
        }

        // Emit content
        for node in nodes {
            self.emit_node_content(node, output, has_expressions);
        }

        output.push("\"\"\")");
        output.newline();
    }

    /// Emit the content of a node as part of a string literal
    fn emit_node_content(&self, node: &Node, output: &mut Output, in_fstring: bool) {
        match node {
            Node::Text(text) => {
                output.push(&text.content);
            }
            Node::Expression(expr) => {
                if in_fstring {
                    let (start, end) = if expr.escape {
                        // Use ‹ESCAPE:{expr}› marker, handled by runtime replace_markers()
                        // The marker includes the braces, so we just push the expression
                        output.push("‹ESCAPE:{");
                        let start = output.position();
                        output.push(&expr.expr);
                        let end = output.position();
                        output.push("}›");
                        (start, end)
                    } else {
                        let start = output.position();
                        output.push("{");
                        output.push(&expr.expr);
                        output.push("}");
                        let end = output.position();
                        (start, end)
                    };

                    // For IDE injection, we want just the expression content (skip the braces)
                    // expr.span includes {x}, so we add 1 to skip { and subtract 1 to skip }
                    let content_start = expr.span.start.byte + 1;
                    let content_end = expr.span.end.byte - 1;

                    output.add_range(Range {
                        range_type: RangeType::Python,
                        source_start: content_start,
                        source_end: content_end,
                        compiled_start: start,
                        compiled_end: end,
                        needs_injection: true,
                    });
                }
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
            self.emit_attribute_content(attr, output, in_fstring);
        }

        if el.self_closing {
            output.push(" />");
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
    }

    /// Check if an attribute name is a boolean HTML attribute
    fn is_boolean_attribute(&self, name: &str) -> bool {
        matches!(
            name,
            "disabled"
                | "checked"
                | "readonly"
                | "required"
                | "autofocus"
                | "autoplay"
                | "controls"
                | "loop"
                | "muted"
                | "selected"
                | "open"
                | "hidden"
                | "async"
                | "defer"
                | "novalidate"
                | "formnovalidate"
                | "ismap"
                | "multiple"
                | "reversed"
                | "scoped"
        )
    }

    /// Emit attribute content as part of a string literal
    fn emit_attribute_content(&self, attr: &Attribute, output: &mut Output, in_fstring: bool) {
        match &attr.kind {
            AttributeKind::Static { name, value } => {
                output.push(" ");
                output.push(name);
                output.push("=\"");
                output.push(value);
                output.push("\"");
            }
            AttributeKind::Dynamic { name, expr, expr_span } => {
                if in_fstring {
                    output.push(" ");
                    output.push(name);

                    // expr_span includes {expr}, skip braces for injection
                    let content_start = expr_span.start.byte + 1;
                    let content_end = expr_span.end.byte - 1;

                    // Use markers for special attribute types
                    // Convert reserved keywords in expressions to safe variable names
                    let safe_expr = self.safe_var_name(expr.trim());

                    if name == "class" {
                        output.push("=‹CLASS:{");
                        let start = output.position();
                        output.push(&safe_expr);
                        let end = output.position();
                        output.add_range(Range {
                            range_type: RangeType::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                        });
                        output.push("}›");
                    } else if name == "style" {
                        output.push("=‹STYLE:{");
                        let start = output.position();
                        output.push(&safe_expr);
                        let end = output.position();
                        output.add_range(Range {
                            range_type: RangeType::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                        });
                        output.push("}›");
                    } else if self.is_boolean_attribute(name) {
                        output.push("=‹BOOL:{");
                        let start = output.position();
                        output.push(&safe_expr);
                        let end = output.position();
                        output.add_range(Range {
                            range_type: RangeType::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                        });
                        output.push("}›");
                    } else {
                        output.push("=\"{");
                        let start = output.position();
                        output.push(&safe_expr);
                        let end = output.position();
                        output.add_range(Range {
                            range_type: RangeType::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                        });
                        output.push("}\"");
                    }
                }
            }
            AttributeKind::Boolean { name } => {
                output.push(" ");
                output.push(name);
            }
            AttributeKind::Shorthand { name, .. } => {
                if in_fstring {
                    output.push(" ");
                    output.push(name);
                    // Use markers for special attribute types
                    // Use safe variable name for reserved keywords inside markers
                    let var_name = self.safe_var_name(name);
                    if name == "class" {
                        output.push("=‹CLASS:{");
                        output.push(&var_name);
                        output.push("}›");
                    } else if name == "style" {
                        output.push("=‹STYLE:{");
                        output.push(&var_name);
                        output.push("}›");
                    } else if name == "data" {
                        output.push("=‹DATA:{");
                        output.push(&var_name);
                        output.push("}›");
                    } else if name == "aria" {
                        output.push("=‹ARIA:{");
                        output.push(&var_name);
                        output.push("}›");
                    } else if self.is_boolean_attribute(name) {
                        output.push("=‹BOOL:{");
                        output.push(&var_name);
                        output.push("}›");
                    } else {
                        // Generic attribute shorthand - treat as spread to support dict expansion
                        output.push("=‹SPREAD:{");
                        output.push(&var_name);
                        output.push("}›");
                    }
                }
            }
            AttributeKind::Spread { expr, .. } => {
                if in_fstring {
                    // Detect special spread types by variable name
                    // Convert reserved keywords to safe variable names
                    let trimmed_expr = expr.trim();
                    let safe_expr = self.safe_var_name(trimmed_expr);

                    if trimmed_expr == "class" {
                        output.push(" class=‹CLASS:{");
                        output.push(&safe_expr);
                        output.push("}›");
                    } else if trimmed_expr == "style" {
                        output.push(" style=‹STYLE:{");
                        output.push(&safe_expr);
                        output.push("}›");
                    } else if trimmed_expr == "data" {
                        output.push(" data=‹DATA:{");
                        output.push(&safe_expr);
                        output.push("}›");
                    } else if trimmed_expr == "aria" {
                        output.push(" aria=‹ARIA:{");
                        output.push(&safe_expr);
                        output.push("}›");
                    } else {
                        // Generic spread - also use safe name for reserved keywords
                        output.push(" ‹SPREAD:{");
                        output.push(&safe_expr);
                        output.push("}›");
                    }
                }
            }
            AttributeKind::SlotAssignment { name, expr, .. } => {
                if let Some(e) = expr {
                    if in_fstring {
                        output.push(" slot:");
                        output.push(name);
                        output.push("=\"{");
                        output.push(e);
                        output.push("}\"");
                    }
                } else {
                    output.push(" slot:");
                    output.push(name);
                }
            }
        }
    }

    fn emit_node(&self, node: &Node, output: &mut Output, indent: usize) {
        match node {
            Node::Text(text) => self.emit_text(text, output, indent),
            Node::Expression(expr) => self.emit_expression(expr, output, indent),
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
        output.push("_parts.append(\"");
        output.push(&escape_string(&text.content));
        output.push("\")");
        output.newline();
    }

    fn emit_expression(&self, expr: &ExpressionNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("_parts.append(");
        if expr.escape {
            output.push("escape(");
        }
        output.push(&expr.expr);
        if expr.escape {
            output.push(")");
        }
        output.push(")");
        output.newline();
    }

    fn emit_element(&self, el: &ElementNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("_parts.append(\"<");
        output.push(&el.tag);

        // Emit attributes
        for attr in &el.attributes {
            self.emit_attribute(attr, output);
        }

        if el.self_closing {
            output.push(" />\")");
            output.newline();
        } else {
            output.push(">\")");
            output.newline();

            // Emit children
            for child in &el.children {
                self.emit_node(child, output, indent);
            }

            // Closing tag
            self.indent(output, indent);
            output.push("_parts.append(\"</");
            output.push(&el.tag);
            output.push(">\")");
            output.newline();
        }
    }

    fn emit_attribute(&self, attr: &Attribute, output: &mut Output) {
        match &attr.kind {
            AttributeKind::Static { name, value } => {
                output.push(" ");
                output.push(name);
                output.push("=\\\"");
                output.push(&escape_string(value));
                output.push("\\\"");
            }
            AttributeKind::Dynamic { name, expr, .. } => {
                output.push(" ");
                output.push(name);
                output.push("=\\\"{");
                output.push(expr);
                output.push("}\\\"");
            }
            AttributeKind::Boolean { name } => {
                output.push(" ");
                output.push(name);
            }
            AttributeKind::Shorthand { name, .. } => {
                output.push(" ");
                output.push(name);
                output.push("=\\\"{");
                output.push(name);
                output.push("}\\\"");
            }
            AttributeKind::Spread { expr, .. } => {
                output.push(" {");
                output.push(expr);
                output.push("}");
            }
            AttributeKind::SlotAssignment { name, expr, .. } => {
                if let Some(e) = expr {
                    output.push(" slot:");
                    output.push(name);
                    output.push("=\\\"{");
                    output.push(e);
                    output.push("}\\\"");
                } else {
                    output.push(" slot:");
                    output.push(name);
                }
            }
        }
    }

    fn emit_component(&self, c: &ComponentNode, output: &mut Output, indent: usize) {
        // If component has children, we need to render them first
        let has_children = !c.children.is_empty();

        if has_children {
            // Create a temporary variable to hold children content
            self.indent(output, indent);
            output.push("_child_parts = []");
            output.newline();

            // Emit children into _child_parts
            let refs: Vec<&Node> = c.children.iter().collect();
            // Temporarily swap the output target (we'll render to the same output but using _child_parts)
            for child in &refs {
                match *child {
                    Node::Text(text) => {
                        self.indent(output, indent);
                        output.push("_child_parts.append(\"");
                        output.push(&escape_string(&text.content));
                        output.push("\")");
                        output.newline();
                    }
                    Node::Expression(expr) => {
                        self.indent(output, indent);
                        output.push("_child_parts.append(");
                        if expr.escape {
                            output.push("escape(");
                        }
                        output.push(&expr.expr);
                        if expr.escape {
                            output.push(")");
                        }
                        output.push(")");
                        output.newline();
                    }
                    Node::Element(el) => {
                        // Recursively emit element
                        self.indent(output, indent);
                        output.push("_child_parts.append(\"<");
                        output.push(&el.tag);
                        for attr in &el.attributes {
                            self.emit_attribute(attr, output);
                        }
                        if el.self_closing {
                            output.push(" />\")");
                            output.newline();
                        } else {
                            output.push(">\")");
                            output.newline();
                            // Recursively handle element children (simplified - just emit as text for now)
                            for child_el in &el.children {
                                self.emit_child_node(child_el, output, indent, "_child_parts");
                            }
                            self.indent(output, indent);
                            output.push("_child_parts.append(\"</");
                            output.push(&el.tag);
                            output.push(">\")");
                            output.newline();
                        }
                    }
                    _ => {
                        // For other node types, emit normally but target _child_parts
                        self.emit_child_node(*child, output, indent, "_child_parts");
                    }
                }
            }
        }

        self.indent(output, indent);
        output.push("_parts.append(");
        output.push(&c.name);
        output.push("(");

        // Emit attributes as keyword arguments
        let mut arg_count = 0;
        for attr in &c.attributes {
            if arg_count > 0 {
                output.push(", ");
            }
            match &attr.kind {
                AttributeKind::Static { name, value } => {
                    output.push(name);
                    output.push("=\"");
                    output.push(&escape_string(value));
                    output.push("\"");
                    arg_count += 1;
                }
                AttributeKind::Dynamic { name, expr, .. } => {
                    output.push(name);
                    output.push("=");
                    output.push(expr);
                    arg_count += 1;
                }
                _ => {} // TODO: handle other attribute types
            }
        }

        // Pass children if present
        if has_children {
            if arg_count > 0 {
                output.push(", ");
            }
            output.push("_children=\"\".join(_child_parts)");
        }

        output.push("))");
        output.newline();
    }

    /// Emit a child node into a specified parts array (for component children)
    fn emit_child_node(&self, node: &Node, output: &mut Output, indent: usize, target: &str) {
        match node {
            Node::Text(text) => {
                self.indent(output, indent);
                output.push(target);
                output.push(".append(\"");
                output.push(&escape_string(&text.content));
                output.push("\")");
                output.newline();
            }
            Node::Expression(expr) => {
                self.indent(output, indent);
                output.push(target);
                output.push(".append(");
                if expr.escape {
                    output.push("escape(");
                }
                output.push(&expr.expr);
                if expr.escape {
                    output.push(")");
                }
                output.push(")");
                output.newline();
            }
            Node::Element(el) => {
                self.indent(output, indent);
                output.push(target);
                output.push(".append(\"<");
                output.push(&el.tag);
                for attr in &el.attributes {
                    self.emit_attribute(attr, output);
                }
                if el.self_closing {
                    output.push(" />\")");
                    output.newline();
                } else {
                    output.push(">\")");
                    output.newline();
                    for child in &el.children {
                        self.emit_child_node(child, output, indent, target);
                    }
                    self.indent(output, indent);
                    output.push(target);
                    output.push(".append(\"</");
                    output.push(&el.tag);
                    output.push(">\")");
                    output.newline();
                }
            }
            _ => {
                // For control flow and other nodes, fall back to regular emit
                // This is a simplification - ideally we'd redirect output
            }
        }
    }

    fn emit_fragment(&self, f: &FragmentNode, output: &mut Output, indent: usize) {
        let refs: Vec<&Node> = f.children.iter().collect();
        self.emit_nodes(&refs, output, indent);
    }

    fn emit_slot(&self, s: &SlotNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        if let Some(name) = &s.name {
            output.push("_parts.append(_");
            output.push(name);
            output.push("_children)");
        } else {
            output.push("_parts.append(_children)");
        }
        output.newline();
    }

    fn emit_if(&self, if_node: &IfNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("if ");
        // Remove trailing colon from condition if present (parsing includes it)
        let condition = if_node.condition.trim_end_matches(':').trim();
        output.push(condition);
        output.push(":");
        output.newline();

        if if_node.then_branch.is_empty() {
            self.indent(output, indent + 1);
            output.push("pass");
            output.newline();
        } else {
            let refs: Vec<&Node> = if_node.then_branch.iter().collect();
            self.emit_nodes(&refs, output, indent + 1);
        }

        for (condition, _, body) in &if_node.elif_branches {
            self.indent(output, indent);
            output.push("elif ");
            let condition = condition.trim_end_matches(':').trim();
            output.push(condition);
            output.push(":");
            output.newline();

            if body.is_empty() {
                self.indent(output, indent + 1);
                output.push("pass");
                output.newline();
            } else {
                let refs: Vec<&Node> = body.iter().collect();
                self.emit_nodes(&refs, output, indent + 1);
            }
        }

        if let Some(else_branch) = &if_node.else_branch {
            self.indent(output, indent);
            output.push("else:");
            output.newline();

            if else_branch.is_empty() {
                self.indent(output, indent + 1);
                output.push("pass");
                output.newline();
            } else {
                let refs: Vec<&Node> = else_branch.iter().collect();
                self.emit_nodes(&refs, output, indent + 1);
            }
        }
    }

    fn emit_for(&self, for_node: &ForNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        if for_node.is_async {
            output.push("async for ");
        } else {
            output.push("for ");
        }
        output.push(&for_node.binding);
        output.push(" in ");

        // Remove trailing colon from iterable if present (parsing includes it)
        let iterable = for_node.iterable.trim_end_matches(':').trim();
        output.push(iterable);
        output.push(":");
        output.newline();

        if for_node.body.is_empty() {
            self.indent(output, indent + 1);
            output.push("pass");
            output.newline();
        } else {
            let refs: Vec<&Node> = for_node.body.iter().collect();
            self.emit_nodes(&refs, output, indent + 1);
        }
    }

    fn emit_match(&self, match_node: &MatchNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("match ");
        // Remove trailing colon from expr if present (parsing includes it)
        let expr = match_node.expr.trim_end_matches(':').trim();
        output.push(expr);
        output.push(":");
        output.newline();

        for case in match_node.cases.iter() {
            self.indent(output, indent + 1);
            output.push("case ");
            // Remove trailing colon from pattern if present
            let pattern = case.pattern.trim_end_matches(':').trim();
            output.push(pattern);
            output.push(":");
            output.newline();

            if case.body.is_empty() {
                self.indent(output, indent + 2);
                output.push("pass");
                output.newline();
            } else {
                let refs: Vec<&Node> = case.body.iter().collect();
                self.emit_nodes(&refs, output, indent + 2);
            }
        }
    }

    fn emit_while(&self, while_node: &WhileNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("while ");
        // Remove trailing colon from condition if present (parsing includes it)
        let condition = while_node.condition.trim_end_matches(':').trim();
        output.push(condition);
        output.push(":");
        output.newline();

        if while_node.body.is_empty() {
            self.indent(output, indent + 1);
            output.push("pass");
            output.newline();
        } else {
            let refs: Vec<&Node> = while_node.body.iter().collect();
            self.emit_nodes(&refs, output, indent + 1);
        }
    }

    fn emit_with(&self, with_node: &WithNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        if with_node.is_async {
            output.push("async with ");
        } else {
            output.push("with ");
        }
        // Remove trailing colon from items if present (parsing includes it)
        let items = with_node.items.trim_end_matches(':').trim();
        output.push(items);
        output.push(":");
        output.newline();

        if with_node.body.is_empty() {
            self.indent(output, indent + 1);
            output.push("pass");
            output.newline();
        } else {
            let refs: Vec<&Node> = with_node.body.iter().collect();
            self.emit_nodes(&refs, output, indent + 1);
        }
    }

    fn emit_try(&self, try_node: &TryNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("try:");
        output.newline();

        if try_node.body.is_empty() {
            self.indent(output, indent + 1);
            output.push("pass");
            output.newline();
        } else {
            let refs: Vec<&Node> = try_node.body.iter().collect();
            self.emit_nodes(&refs, output, indent + 1);
        }

        for except in &try_node.except_clauses {
            self.indent(output, indent);
            output.push("except");
            if let Some(exception) = &except.exception {
                output.push(" ");
                output.push(exception);
            }
            output.push(":");
            output.newline();

            if except.body.is_empty() {
                self.indent(output, indent + 1);
                output.push("pass");
                output.newline();
            } else {
                let refs: Vec<&Node> = except.body.iter().collect();
                self.emit_nodes(&refs, output, indent + 1);
            }
        }

        if let Some(else_clause) = &try_node.else_clause {
            self.indent(output, indent);
            output.push("else:");
            output.newline();

            if else_clause.is_empty() {
                self.indent(output, indent + 1);
                output.push("pass");
                output.newline();
            } else {
                let refs: Vec<&Node> = else_clause.iter().collect();
                self.emit_nodes(&refs, output, indent + 1);
            }
        }

        if let Some(finally_clause) = &try_node.finally_clause {
            self.indent(output, indent);
            output.push("finally:");
            output.newline();

            if finally_clause.is_empty() {
                self.indent(output, indent + 1);
                output.push("pass");
                output.newline();
            } else {
                let refs: Vec<&Node> = finally_clause.iter().collect();
                self.emit_nodes(&refs, output, indent + 1);
            }
        }
    }

    fn emit_statement(&self, stmt: &StatementNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);

        // Handle reserved keywords by prefixing with underscore
        // This handles cases like: class = [...], type = ..., etc.
        let statement = stmt.stmt
            .replace("class =", "_class =")
            .replace("class=", "_class=")
            .replace("type =", "_type =")
            .replace("type=", "_type=");

        output.push(&statement);
        output.newline();
    }

    fn emit_definition(&self, def: &DefinitionNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push(&def.signature);
        output.newline();

        if def.body.is_empty() {
            self.indent(output, indent + 1);
            output.push("pass");
            output.newline();
        } else {
            let refs: Vec<&Node> = def.body.iter().collect();
            self.emit_nodes(&refs, output, indent + 1);
        }
    }

    fn emit_import(&self, import: &ImportNode, output: &mut Output, _indent: usize) {
        output.push(&import.stmt);
        output.newline();
    }

    fn emit_decorator(&self, dec: &DecoratorNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push(&dec.decorator);
        output.newline();
    }

    fn indent(&self, output: &mut Output, level: usize) {
        for _ in 0..level {
            output.push("    ");
        }
    }
}

impl Default for PythonGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for PythonGenerator {
    fn generate(&self, ast: &Ast, metadata: &crate::transform::TransformMetadata, options: &GenerateOptions) -> GenerateResult {
        let mut output = Output::new();

        // Collect parameters and imports from AST
        let mut parameters = Vec::new();
        let mut imports = Vec::new();
        let mut body_nodes = Vec::new();

        for node in &ast.nodes {
            match node {
                Node::Parameter(param) => parameters.push(param),
                Node::Import(import) => imports.push(import),
                _ => body_nodes.push(node),
            }
        }

        // Emit user imports
        for import in &imports {
            output.push(&import.stmt);
            output.newline();
        }

        // Emit function signature with parameters
        let func_name = options.function_name.as_deref()
            .map(|name| to_pascal_case(name))
            .unwrap_or_else(|| "Render".to_string());

        // Add async if needed
        if metadata.is_async {
            output.push("async def ");
        } else {
            output.push("def ");
        }
        output.push(&func_name);
        output.push("(");

        // Emit parameters
        let mut param_count = 0;
        for param in &parameters {
            if param_count > 0 {
                output.push(", ");
            }
            let param_start = output.position();
            output.push(&param.name);
            if let Some(type_hint) = &param.type_hint {
                output.push(": ");
                output.push(type_hint);
            }
            if let Some(default) = &param.default {
                output.push(" = ");
                output.push(default);
            }
            let param_end = output.position();

            // Add range for parameter (maps source parameter to compiled signature)
            // Parameters in frontmatter don't need IDE injection (they're already Python)
            output.add_range(Range {
                range_type: RangeType::Python,
                source_start: param.span.start.byte,
                source_end: param.span.end.byte,
                compiled_start: param_start,
                compiled_end: param_end,
                needs_injection: false,
            });

            param_count += 1;
        }

        // Add slot parameters (children, named slots) as keyword-only
        if !metadata.slots_used.is_empty() {
            // Add keyword-only marker
            if param_count > 0 {
                output.push(", *");
            } else {
                output.push("*");
            }

            // Sort slot names for deterministic output
            let mut sorted_slots: Vec<_> = metadata.slots_used.iter().collect();
            sorted_slots.sort();

            // Add each slot parameter
            for slot_name in sorted_slots {
                output.push(", ");
                // Default slot uses _children, named slots use _{name}_children
                if slot_name.is_empty() {
                    output.push("_children: str = \"\"");
                } else {
                    output.push("_");
                    output.push(slot_name);
                    output.push("_children: str = \"\"");
                }
            }
        }

        output.push(") -> str:");
        output.newline();

        // Initialize _parts
        self.indent(&mut output, 1);
        output.push("_parts = []");
        output.newline();

        // Emit body
        self.emit_nodes(&body_nodes, &mut output, 1);

        // Return joined parts (we'll add replace_markers wrapper during post-processing if needed)
        self.indent(&mut output, 1);
        output.push("return \"\".join(_parts)");
        output.newline();

        let (mut code, mappings, tracked_ranges) = output.finish();

        // Track the import line offset for adjusting ranges
        let mut import_offset = 0;

        // Add replace_markers import and wrap return if markers are present
        if code.contains('‹') {
            // Add helpers based on what's used
            let mut helpers_to_import = Vec::new();

            // Check which helpers are used in metadata
            if metadata.helpers_used.contains("escape") || code.contains("escape(") {
                helpers_to_import.push("escape");
            }
            if metadata.helpers_used.contains("safe") {
                helpers_to_import.push("safe");
            }
            if metadata.helpers_used.contains("render_class") {
                helpers_to_import.push("render_class");
            }
            if metadata.helpers_used.contains("render_style") {
                helpers_to_import.push("render_style");
            }
            if metadata.helpers_used.contains("render_attr") {
                helpers_to_import.push("render_attr");
            }
            if metadata.helpers_used.contains("render_data") {
                helpers_to_import.push("render_data");
            }
            if metadata.helpers_used.contains("render_aria") {
                helpers_to_import.push("render_aria");
            }
            if metadata.helpers_used.contains("spread_attrs") {
                helpers_to_import.push("spread_attrs");
            }

            // Always add replace_markers when markers are present
            helpers_to_import.push("replace_markers");

            if !helpers_to_import.is_empty() {
                let import_line = format!("from hyper import {}\n\n", helpers_to_import.join(", "));

                // Find the position after user imports
                if let Some(def_pos) = code.find("def ") {
                    code.insert_str(def_pos, &import_line);
                    import_offset = import_line.len();
                } else {
                    // Fallback: prepend at the start
                    code.insert_str(0, &import_line);
                    import_offset = import_line.len();
                }
            }

            // Wrap return statement with replace_markers
            code = code.replace(
                "return \"\".join(_parts)",
                "return replace_markers(\"\".join(_parts))"
            );
        } else {
            // No markers, just add regular helper imports
            let mut helpers_to_import = Vec::new();

            if metadata.helpers_used.contains("escape") || code.contains("escape(") {
                helpers_to_import.push("escape");
            }
            if metadata.helpers_used.contains("safe") {
                helpers_to_import.push("safe");
            }
            if metadata.helpers_used.contains("render_class") {
                helpers_to_import.push("render_class");
            }
            if metadata.helpers_used.contains("render_style") {
                helpers_to_import.push("render_style");
            }
            if metadata.helpers_used.contains("render_attr") {
                helpers_to_import.push("render_attr");
            }
            if metadata.helpers_used.contains("render_data") {
                helpers_to_import.push("render_data");
            }
            if metadata.helpers_used.contains("render_aria") {
                helpers_to_import.push("render_aria");
            }
            if metadata.helpers_used.contains("spread_attrs") {
                helpers_to_import.push("spread_attrs");
            }

            if !helpers_to_import.is_empty() {
                let import_line = format!("from hyper import {}\n\n", helpers_to_import.join(", "));

                if let Some(def_pos) = code.find("def ") {
                    code.insert_str(def_pos, &import_line);
                    import_offset = import_line.len();
                } else {
                    code.insert_str(0, &import_line);
                    import_offset = import_line.len();
                }
            }
        }

        // Compute injection ranges and injections using the analyzer (if requested)
        let (ranges, injections) = if options.include_ranges {
            // Adjust tracked ranges by the import line offset
            let adjusted_ranges: Vec<crate::generate::Range> = tracked_ranges.into_iter().map(|mut r| {
                r.compiled_start += import_offset;
                r.compiled_end += import_offset;
                r
            }).collect();

            let analyzer = super::InjectionAnalyzer::new();
            analyzer.analyze(ast, &code, adjusted_ranges)
        } else {
            (Vec::new(), Vec::new())
        };

        GenerateResult {
            code,
            mappings,
            ranges,
            injections,
        }
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
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
