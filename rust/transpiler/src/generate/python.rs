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

    /// Check if a list of nodes contains only whitespace/newline text (no real content)
    fn is_effectively_empty(&self, nodes: &[&Node]) -> bool {
        nodes.iter().all(|node| match node {
            Node::Text(t) => t.content.trim().is_empty(),
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

    /// Check if content contains markers that need replace_markers()
    fn content_has_markers(&self, nodes: &[&Node]) -> bool {
        for node in nodes {
            if self.node_has_markers(node) {
                return true;
            }
        }
        false
    }

    /// Check if a single node contains markers
    fn node_has_markers(&self, node: &Node) -> bool {
        match node {
            Node::Expression(expr) => expr.escape, // Escaped expressions use markers
            Node::Element(el) => {
                // Check attributes for markers (class, style, bool, spread, template)
                el.attributes.iter().any(|attr| {
                    matches!(attr.kind,
                        AttributeKind::Dynamic { ref name, .. } if name == "class" || name == "style" || self.is_boolean_attribute(name))
                    || matches!(attr.kind, AttributeKind::Shorthand { .. } | AttributeKind::Spread { .. } | AttributeKind::Template { .. })
                }) || el.children.iter().any(|child| self.node_has_markers(child))
            }
            _ => false,
        }
    }

    /// Emit consecutive text/expression/element nodes as a single yield statement
    fn emit_combined_nodes(&self, nodes: &[&Node], output: &mut Output, indent: usize) {
        // Check if any node contains expressions (recursively)
        let has_expressions = nodes.iter().any(|node| self.node_has_expressions(node));
        // Check if content has markers that need replace_markers()
        let has_markers = self.content_has_markers(nodes);

        // Collect content to a temporary buffer to analyze it and capture ranges
        let mut content_output = Output::new();
        for node in nodes {
            self.emit_node_content(node, &mut content_output, has_expressions);
        }
        let ranges = content_output.take_ranges();
        let (content, _, _) = content_output.finish();

        // Calculate how much leading content we're trimming (needed for range adjustment)
        let trimmed = content.trim_start_matches(|c| c == '\n' || c == ' ');
        let leading_trimmed = content.len() - trimmed.len();
        let leading_trimmed_utf16 = content[..leading_trimmed].encode_utf16().count();
        // Only trim ONE trailing newline (the line ending), preserve any extras (blank lines)
        let trimmed = trimmed.strip_suffix('\n').unwrap_or(trimmed);
        // Count remaining trailing newlines (these are blank lines to preserve)
        let trailing_blank_lines = trimmed.chars().rev().take_while(|&c| c == '\n').count();
        let trimmed = trimmed.trim_end_matches('\n');

        // If content is empty after trimming, check if we need to preserve blank lines
        if trimmed.is_empty() {
            // If original content had newlines (blank lines), emit a blank line
            if content.contains('\n') {
                output.newline();
            }
            return;
        }

        self.indent(output, indent);

        // Determine if content is multiline
        let is_multiline = trimmed.contains('\n');

        // Build the yield statement
        if has_markers {
            if is_multiline {
                output.push("yield replace_markers(f\"\"\"\\");
                output.newline();
            } else {
                output.push("yield replace_markers(f\"\"\"");
            }
        } else if has_expressions {
            if is_multiline {
                output.push("yield f\"\"\"\\");
                output.newline();
            } else {
                output.push("yield f\"\"\"");
            }
        } else {
            if is_multiline {
                output.push("yield \"\"\"\\");
                output.newline();
            } else {
                output.push("yield \"\"\"");
            }
        }

        // Get position before emitting content (for range offset calculation)
        let content_start_pos = output.position();

        // Emit the trimmed content
        output.push(trimmed);

        // Transfer ranges from temp buffer, adjusting for:
        // 1. The position where content starts in main output
        // 2. The leading content that was trimmed
        // Note: ranges use signed arithmetic to handle the subtraction
        let offset = content_start_pos as isize - leading_trimmed_utf16 as isize;
        for mut range in ranges {
            range.compiled_start = (range.compiled_start as isize + offset) as usize;
            range.compiled_end = (range.compiled_end as isize + offset) as usize;
            output.add_range(range);
        }

        if has_markers {
            output.push("\"\"\")");
        } else {
            output.push("\"\"\"");
        }
        output.newline();

        // Emit preserved blank lines
        for _ in 0..trailing_blank_lines {
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
            Node::Expression(expr) => {
                if in_fstring {
                    let (start, end) = if expr.escape {
                        // Use ‹ESCAPE:{expr}› marker, handled by runtime replace_markers()
                        // Track {expr} including braces for IDE highlighting
                        output.push("‹ESCAPE:");
                        let start = output.position();
                        output.push("{");
                        output.push(&expr.expr);
                        output.push("}");
                        let end = output.position();
                        output.push("›");
                        (start, end)
                    } else {
                        let start = output.position();
                        output.push("{");
                        output.push(&expr.expr);
                        output.push("}");
                        let end = output.position();
                        (start, end)
                    };

                    // For IDE injection, include the braces so they get f-string highlighting
                    let content_start = expr.span.start.byte;
                    let content_end = expr.span.end.byte;

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
            AttributeKind::Template { name, value } => {
                if in_fstring {
                    output.push(" ");
                    output.push(name);
                    output.push("=\"");
                    // Parse and emit the template value, converting {expr} to ‹ESCAPE:{expr}›
                    output.push(&self.convert_template_expressions(value));
                    output.push("\"");
                }
            }
        }
    }

    /// Convert {expr} in template string to ‹ESCAPE:{expr}› for runtime processing
    fn convert_template_expressions(&self, template: &str) -> String {
        let mut result = String::new();
        let mut chars = template.chars().peekable();

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
                // Emit as ESCAPE marker
                result.push_str("‹ESCAPE:{");
                result.push_str(&expr);
                result.push_str("}›");
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
        if expr.escape {
            output.push("yield escape(");
            output.push(&expr.expr);
            output.push(")");
        } else {
            output.push("yield str(");
            output.push(&expr.expr);
            output.push(")");
        }
        output.newline();
    }

    fn emit_element(&self, el: &ElementNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("yield \"<");
        output.push(&el.tag);

        // Emit attributes
        for attr in &el.attributes {
            self.emit_attribute(attr, output);
        }

        if el.self_closing {
            output.push(" />\"");
            output.newline();
        } else {
            output.push(">\"");
            output.newline();

            // Emit children using emit_nodes for proper grouping
            let refs: Vec<&Node> = el.children.iter().collect();
            self.emit_nodes(&refs, output, indent);

            // Closing tag
            self.indent(output, indent);
            output.push("yield \"</");
            output.push(&el.tag);
            output.push(">\"");
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
            AttributeKind::Template { name, value } => {
                output.push(" ");
                output.push(name);
                output.push("=\\\"");
                // Convert {expr} to f-string syntax with escaping
                output.push(&self.convert_template_expressions(value));
                output.push("\\\"");
            }
        }
    }

    /// Generate a safe function name from a component name
    fn component_to_func_name(&self, name: &str) -> String {
        // Convert PascalCase to snake_case and prefix with _
        // Skip non-identifier characters (brackets, quotes, dots, etc.)
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
                // Non-identifier character acts as a separator
                if !prev_was_separator && i > 0 && !result.ends_with('_') {
                    result.push('_');
                }
                prev_was_separator = true;
            }
        }
        // Trim trailing underscore from separators
        while result.ends_with('_') && result.len() > 1 {
            result.pop();
        }
        result
    }

    fn emit_component(&self, c: &ComponentNode, output: &mut Output, indent: usize) {
        let has_children = !c.children.is_empty();

        if has_children {
            // Generate inner function name from component name
            let func_name = self.component_to_func_name(&c.name);

            // Emit comment for opening tag
            self.indent(output, indent);
            output.push("# <{");
            output.push(&c.name);
            output.push("}>");
            output.newline();

            // Emit inner function definition
            self.indent(output, indent);
            output.push("def ");
            output.push(&func_name);
            output.push("():");
            output.newline();

            // Emit children inside the inner function
            self.emit_body_or_pass(&c.children, output, indent + 1);

            // Emit yield from with component call
            self.indent(output, indent);
            output.push("yield from ");
            output.push(&c.name);
            output.push("(");
            output.push(&func_name);
            output.push("()");

            // Emit attributes as keyword arguments
            for attr in &c.attributes {
                output.push(", ");
                match &attr.kind {
                    AttributeKind::Static { name, value } => {
                        output.push(name);
                        output.push("=\"");
                        output.push(&escape_string(value));
                        output.push("\"");
                    }
                    AttributeKind::Dynamic { name, expr, .. } => {
                        output.push(name);
                        output.push("=");
                        output.push(expr);
                    }
                    _ => {}
                }
            }

            output.push(")");
            output.newline();

            // Emit comment for closing tag
            self.indent(output, indent);
            output.push("# </{");
            output.push(&c.name);
            output.push("}>");
            output.newline();
        } else {
            // No children - simple yield from
            self.indent(output, indent);
            output.push("yield from ");
            output.push(&c.name);
            output.push("(");

            // Emit attributes as keyword arguments
            let mut first = true;
            for attr in &c.attributes {
                if !first {
                    output.push(", ");
                }
                first = false;
                match &attr.kind {
                    AttributeKind::Static { name, value } => {
                        output.push(name);
                        output.push("=\"");
                        output.push(&escape_string(value));
                        output.push("\"");
                    }
                    AttributeKind::Dynamic { name, expr, .. } => {
                        output.push(name);
                        output.push("=");
                        output.push(expr);
                    }
                    _ => {}
                }
            }

            output.push(")");
            output.newline();
        }
    }

    fn emit_fragment(&self, f: &FragmentNode, output: &mut Output, indent: usize) {
        let refs: Vec<&Node> = f.children.iter().collect();
        self.emit_nodes(&refs, output, indent);
    }

    fn emit_slot(&self, s: &SlotNode, output: &mut Output, indent: usize) {
        // Emit conditional yield from for slot content
        let slot_var = if let Some(name) = &s.name {
            format!("_{}", name)
        } else {
            "_content".to_string()
        };

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
    }

    fn emit_if(&self, if_node: &IfNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("if ");
        // Remove trailing colon from condition if present (parsing includes it)
        let condition = if_node.condition.trim_end_matches(':').trim();
        let cond_start = output.position();
        output.push(condition);
        let cond_end = output.position();
        // Track condition for Python injection
        // Adjust source_end to match actual content (trim trailing : and whitespace)
        let source_end = if_node.condition_span.start.byte + condition.len();
        output.add_range(Range {
            range_type: RangeType::Python,
            source_start: if_node.condition_span.start.byte,
            source_end,
            compiled_start: cond_start,
            compiled_end: cond_end,
            needs_injection: true,
        });
        output.push(":");
        output.newline();

        self.emit_body_or_pass(&if_node.then_branch, output, indent + 1);

        for (condition, condition_span, body) in &if_node.elif_branches {
            self.indent(output, indent);
            output.push("elif ");
            let condition = condition.trim_end_matches(':').trim();
            let cond_start = output.position();
            output.push(condition);
            let cond_end = output.position();
            let source_end = condition_span.start.byte + condition.len();
            output.add_range(Range {
                range_type: RangeType::Python,
                source_start: condition_span.start.byte,
                source_end,
                compiled_start: cond_start,
                compiled_end: cond_end,
                needs_injection: true,
            });
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
        output.push(&for_node.binding);
        output.push(" in ");

        // Remove trailing colon from iterable if present (parsing includes it)
        let iterable = for_node.iterable.trim_end_matches(':').trim();
        let iter_start = output.position();
        output.push(iterable);
        let iter_end = output.position();
        let source_end = for_node.iterable_span.start.byte + iterable.len();
        output.add_range(Range {
            range_type: RangeType::Python,
            source_start: for_node.iterable_span.start.byte,
            source_end,
            compiled_start: iter_start,
            compiled_end: iter_end,
            needs_injection: true,
        });
        output.push(":");
        output.newline();

        self.emit_body_or_pass(&for_node.body, output, indent + 1);
    }

    fn emit_match(&self, match_node: &MatchNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("match ");
        // Remove trailing colon from expr if present (parsing includes it)
        let expr = match_node.expr.trim_end_matches(':').trim();
        let expr_start = output.position();
        output.push(expr);
        let expr_end = output.position();
        let source_end = match_node.expr_span.start.byte + expr.len();
        output.add_range(Range {
            range_type: RangeType::Python,
            source_start: match_node.expr_span.start.byte,
            source_end,
            compiled_start: expr_start,
            compiled_end: expr_end,
            needs_injection: true,
        });
        output.push(":");
        output.newline();

        for case in match_node.cases.iter() {
            self.indent(output, indent + 1);
            output.push("case ");
            // Remove trailing colon from pattern if present
            let pattern = case.pattern.trim_end_matches(':').trim();
            let pat_start = output.position();
            output.push(pattern);
            let pat_end = output.position();
            let source_end = case.pattern_span.start.byte + pattern.len();
            output.add_range(Range {
                range_type: RangeType::Python,
                source_start: case.pattern_span.start.byte,
                source_end,
                compiled_start: pat_start,
                compiled_end: pat_end,
                needs_injection: true,
            });
            output.push(":");
            output.newline();

            self.emit_body_or_pass(&case.body, output, indent + 2);
        }
    }

    fn emit_while(&self, while_node: &WhileNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push("while ");
        // Remove trailing colon from condition if present (parsing includes it)
        let condition = while_node.condition.trim_end_matches(':').trim();
        let cond_start = output.position();
        output.push(condition);
        let cond_end = output.position();
        let source_end = while_node.condition_span.start.byte + condition.len();
        output.add_range(Range {
            range_type: RangeType::Python,
            source_start: while_node.condition_span.start.byte,
            source_end,
            compiled_start: cond_start,
            compiled_end: cond_end,
            needs_injection: true,
        });
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
        // Remove trailing colon from items if present (parsing includes it)
        let items = with_node.items.trim_end_matches(':').trim();
        let items_start = output.position();
        output.push(items);
        let items_end = output.position();
        // Calculate source_end based on trimmed content length to avoid including the colon
        let source_end = with_node.items_span.start.byte + items.len();
        output.add_range(Range {
            range_type: RangeType::Python,
            source_start: with_node.items_span.start.byte,
            source_end,
            compiled_start: items_start,
            compiled_end: items_end,
            needs_injection: true,
        });
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
                output.push(exception);
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

        // Rename Python reserved keywords used as variable names in assignments.
        // This matches how shorthand attributes rename {class} → _class, {type} → _type.
        let owned_statement;
        let statement = if stmt.stmt.starts_with("class ") || stmt.stmt.starts_with("class=") {
            owned_statement = format!("_{}", &stmt.stmt);
            &owned_statement
        } else if stmt.stmt.starts_with("type ") || stmt.stmt.starts_with("type=") {
            owned_statement = format!("_{}", &stmt.stmt);
            &owned_statement
        } else {
            &stmt.stmt
        };

        // For multiline statements, add indent to each continuation line
        if statement.contains('\n') {
            let indent_str = "    ".repeat(indent);
            let lines: Vec<&str> = statement.split('\n').collect();
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    output.push(&indent_str);
                }
                output.push(line);
                if i < lines.len() - 1 {
                    output.newline();
                }
            }
        } else {
            output.push(statement);
        }
        output.newline();
    }

    fn emit_definition(&self, def: &DefinitionNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        output.push(&def.signature);
        output.newline();

        self.emit_body_or_pass(&def.body, output, indent + 1);
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

        // Collect parameters, imports, decorators, and body from AST
        let mut parameters = Vec::new();
        let mut imports = Vec::new();
        let mut decorators = Vec::new();
        let mut body_nodes = Vec::new();

        // First pass: identify which decorators lead to definitions
        // so we can correctly handle decorator-definition grouping
        let mut decorator_leads_to_def = vec![false; ast.nodes.len()];
        let mut whitespace_in_decorator_chain = vec![false; ast.nodes.len()];

        for (i, node) in ast.nodes.iter().enumerate() {
            if matches!(node, Node::Decorator(_)) {
                let mut found_def = false;
                for j in (i + 1)..ast.nodes.len() {
                    match &ast.nodes[j] {
                        Node::Decorator(_) | Node::Comment(_) => continue,
                        Node::Text(t) if t.content.trim().is_empty() => continue,
                        Node::Definition(_) => { found_def = true; break; }
                        _ => break,
                    }
                }
                decorator_leads_to_def[i] = found_def;

                // Mark whitespace text nodes between this decorator and the next
                // decorator/definition as part of the decorator chain (suppress them)
                if found_def {
                    for j in (i + 1)..ast.nodes.len() {
                        match &ast.nodes[j] {
                            Node::Text(t) if t.content.trim().is_empty() => {
                                whitespace_in_decorator_chain[j] = true;
                            }
                            Node::Decorator(_) | Node::Comment(_) => continue,
                            _ => break,
                        }
                    }
                }
            }
        }

        for (i, node) in ast.nodes.iter().enumerate() {
            match node {
                Node::Parameter(param) => parameters.push(param),
                Node::Import(import) => imports.push(import),
                Node::Decorator(dec) => {
                    if decorator_leads_to_def[i] {
                        body_nodes.push(node);
                    } else {
                        decorators.push(dec);
                    }
                }
                // Skip whitespace text that's between a decorator and its definition
                Node::Text(t) if whitespace_in_decorator_chain[i] && t.content.trim().is_empty() => {}
                _ => body_nodes.push(node),
            }
        }

        // Emit user imports
        for import in &imports {
            output.push(&import.stmt);
            output.newline();
        }

        // Note: orphaned decorators (not attached to inner defs) are applied to
        // the outer template function, emitted later alongside @html in the
        // import block insertion step.

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

        // Determine if we have slots (for _content parameter)
        let has_default_slot = metadata.slots_used.contains("");
        let has_named_slots = metadata.slots_used.iter().any(|s| !s.is_empty());

        // Separate regular params from **kwargs
        // Note: *args is rejected at parse time - hyper uses keyword-only params
        let mut regular_params: Vec<_> = Vec::new();
        let mut star_star_kwargs: Option<&ParameterNode> = None;

        for param in &parameters {
            if param.name.starts_with("**") {
                star_star_kwargs = Some(param);
            } else {
                regular_params.push(param);
            }
        }

        // Emit _content parameter first if default slot is used
        let mut param_count = 0;
        if has_default_slot {
            output.push("_content: Iterable[str] | None = None");
            param_count += 1;
        }

        // Add keyword-only marker if we have user parameters
        // All hyper params are keyword-only (*, prefix)
        if !regular_params.is_empty() {
            if param_count > 0 {
                output.push(", *, ");
            } else {
                output.push("*, ");
            }
        }

        // Emit regular user parameters
        for (i, param) in regular_params.iter().enumerate() {
            if i > 0 {
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

            output.add_range(Range {
                range_type: RangeType::Python,
                source_start: param.span.start.byte,
                source_end: param.span.end.byte,
                compiled_start: param_start,
                compiled_end: param_end,
                needs_injection: true,
            });
        }

        // Add named slot parameters
        if has_named_slots {
            let mut sorted_slots: Vec<_> = metadata.slots_used.iter()
                .filter(|s| !s.is_empty())
                .collect();
            sorted_slots.sort();

            for slot_name in sorted_slots {
                if param_count > 0 || !regular_params.is_empty() {
                    output.push(", ");
                }
                output.push("_");
                output.push(slot_name);
                output.push(": Iterable[str] | None = None");
            }
        }

        // Emit **kwargs if present
        if let Some(kwargs) = star_star_kwargs {
            if param_count > 0 || !regular_params.is_empty() || has_named_slots {
                output.push(", ");
            }
            output.push(&kwargs.name);
            if let Some(type_hint) = &kwargs.type_hint {
                output.push(": ");
                output.push(type_hint);
            }
        }

        output.push("):");
        output.newline();

        // Emit body (using yield instead of _parts)
        if body_nodes.is_empty() || self.is_effectively_empty(&body_nodes) {
            self.indent(&mut output, 1);
            output.push("pass");
            output.newline();
        } else {
            self.emit_nodes(&body_nodes, &mut output, 1);
        }

        let (mut code, mappings, tracked_ranges) = output.finish();

        // Determine if we need Iterable import (for _content parameter)
        let has_default_slot = metadata.slots_used.contains("");
        let has_named_slots = metadata.slots_used.iter().any(|s| !s.is_empty());
        let needs_iterable = has_default_slot || has_named_slots;

        // Build imports
        let mut hyper_imports = vec!["html"];

        // Add replace_markers if markers are present
        if code.contains('‹') {
            hyper_imports.push("replace_markers");
        }

        // Add other helpers based on metadata (only if actually called in code)
        // Note: escape() is never directly called - we use ‹ESCAPE:...› markers
        if metadata.helpers_used.contains("safe") {
            hyper_imports.push("safe");
        }

        // Detect typing constructs needed from parameter type hints
        let mut typing_imports: Vec<&str> = Vec::new();
        let all_type_hints: String = parameters.iter()
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
            import_lines.push_str(&format!("from typing import {}\n", typing_imports.join(", ")));
        }

        // Add Iterable import if needed
        if needs_iterable {
            import_lines.push_str("from collections.abc import Iterable\n");
        }

        // Add hyper imports
        import_lines.push_str(&format!("from hyper import {}\n", hyper_imports.join(", ")));
        import_lines.push_str("\n\n");  // Two blank lines before function (PEP 8)

        // Add user decorators for the outer template function (before @html)
        for dec in &decorators {
            import_lines.push_str(&dec.decorator);
            import_lines.push('\n');
        }

        // Add @html decorator
        import_lines.push_str("@html\n");

        // Insert imports before function definition
        // Search for "async def" first to avoid matching "def" inside "async def"
        let import_offset = if let Some(def_pos) = code.find("async def ").or_else(|| code.find("def ")) {
            code.insert_str(def_pos, &import_lines);
            import_lines.len()
        } else {
            code.insert_str(0, &import_lines);
            import_lines.len()
        };

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
