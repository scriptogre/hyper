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
            "class" => "class_".to_string(),
            "type" => "type_".to_string(),
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

                // Check if the next node is an inline comment (same source line as content)
                let trailing_comment = if j < nodes.len() {
                    if let Node::Comment(c) = nodes[j] {
                        if c.inline { Some(c) } else { None }
                    } else { None }
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
                // Check if element has dynamic attributes (including spreads) or expression children
                el.attributes.iter().any(|attr| {
                    !matches!(attr.kind, AttributeKind::Static { .. } | AttributeKind::Boolean { .. })
                })
                    || el.children.iter().any(|child| self.node_has_expressions(child))
            }
            _ => false,
        }
    }

    /// Emit consecutive text/expression/element nodes as a single yield statement.
    /// If trailing_comment is Some, the comment is appended inline after the closing `"""`.
    fn emit_combined_nodes(&self, nodes: &[&Node], output: &mut Output, indent: usize, trailing_comment: Option<&CommentNode>) {
        // Check if any node contains expressions (recursively)
        let has_expressions = nodes.iter().any(|node| self.node_has_expressions(node));

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
        if has_expressions {
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

        output.push("\"\"\"");
        if let Some(comment) = trailing_comment {
            output.push("  ");
            output.push(&comment.text);
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
                    let has_format_extras = expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
                    let (start, end) = if has_format_extras {
                        // Format spec, conversion, or debug — emit raw (no escape wrapper)
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
                    } else if expr.escape {
                        // Use direct escape() call inside f-string
                        // Track just the expression text for IDE highlighting
                        output.push("{escape(");
                        let start = output.position();
                        output.push(&expr.expr);
                        let end = output.position();
                        output.push(")}");
                        (start, end)
                    } else {
                        let start = output.position();
                        output.push("{");
                        output.push(&expr.expr);
                        output.push("}");
                        let end = output.position();
                        (start, end)
                    };

                    // Source range excludes braces — just the inner expression
                    let content_start = expr.span.start.byte + 1; // skip '{'
                    let content_end = expr.span.end.byte - 1;     // skip '}'

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

        // Add HTML injection ranges for this element's static HTML parts
        self.add_html_ranges(el, output);
    }

    /// Add HTML ranges for an element's opening and closing tags.
    /// The opening tag span (`el.span`) covers `<tag attrs>` or `<tag attrs />`.
    /// The closing tag span (`el.close_span`) covers `</tag>`.
    /// We create HTML ranges for the static parts, skipping over expression spans.
    fn add_html_ranges(&self, el: &ElementNode, output: &mut Output) {
        // Collect expression spans (exclusive end) within the opening tag.
        // Dynamic spans already use exclusive end (past '}').
        // Shorthand/Spread/SlotAssignment spans end AT '}', so we +1 for exclusive end.
        let mut expr_spans = Vec::new();
        for attr in &el.attributes {
            match &attr.kind {
                AttributeKind::Dynamic { expr_span, .. } => {
                    // Include the = sign before { so virtual HTML sees a boolean attr
                    let gap_start = expr_span.start.byte.saturating_sub(1);
                    expr_spans.push((gap_start, expr_span.end.byte));
                }
                AttributeKind::Shorthand { expr_span, .. } => {
                    expr_spans.push((expr_span.start.byte, expr_span.end.byte + 1));
                }
                AttributeKind::Spread { expr_span, .. } => {
                    expr_spans.push((expr_span.start.byte, expr_span.end.byte + 1));
                }
                AttributeKind::SlotAssignment { expr_span: Some(span), .. } => {
                    // Include the = sign before { so virtual HTML sees a boolean attr
                    let gap_start = span.start.byte.saturating_sub(1);
                    expr_spans.push((gap_start, span.end.byte + 1));
                }
                AttributeKind::Template { name, value } => {
                    // Walk value to find {expr} positions, exclude them from HTML ranges
                    let value_start_byte = attr.span.start.byte + name.len() + 2;
                    let mut byte_offset = 0;
                    let mut chars = value.chars().peekable();
                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            let gap_start = value_start_byte + byte_offset;
                            byte_offset += ch.len_utf8();
                            let mut depth = 1;
                            while let Some(inner) = chars.next() {
                                byte_offset += inner.len_utf8();
                                if inner == '{' { depth += 1; }
                                else if inner == '}' {
                                    depth -= 1;
                                    if depth == 0 { break; }
                                }
                            }
                            let gap_end = value_start_byte + byte_offset;
                            expr_spans.push((gap_start, gap_end));
                        } else {
                            byte_offset += ch.len_utf8();
                        }
                    }
                }
                _ => {}
            }
        }

        // Sort by start position
        expr_spans.sort_by_key(|s| s.0);

        // Create HTML ranges for the gaps between expressions within the opening tag
        let tag_start = el.span.start.byte;
        let tag_end = el.span.end.byte;
        let mut pos = tag_start;

        for (expr_start, expr_end) in &expr_spans {
            if *expr_start > pos && *expr_start <= tag_end {
                output.add_range(Range {
                    range_type: RangeType::Html,
                    source_start: pos,
                    source_end: *expr_start,
                    compiled_start: 0,
                    compiled_end: 0,
                    needs_injection: true,
                });
            }
            if *expr_end > pos {
                pos = *expr_end;
            }
        }

        // Remaining static part of opening tag
        if pos < tag_end {
            output.add_range(Range {
                range_type: RangeType::Html,
                source_start: pos,
                source_end: tag_end,
                compiled_start: 0,
                compiled_end: 0,
                needs_injection: true,
            });
        }

        // Closing tag range (e.g. </div>)
        if let Some(close_span) = &el.close_span {
            output.add_range(Range {
                range_type: RangeType::Html,
                source_start: close_span.start.byte,
                source_end: close_span.end.byte,
                compiled_start: 0,
                compiled_end: 0,
                needs_injection: true,
            });
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
                output.push(&escape_html_attr_quotes(value));
                output.push("\"");
            }
            AttributeKind::Dynamic { name, expr, expr_span } => {
                if in_fstring {
                    // expr_span includes {expr}, skip braces for injection
                    let content_start = expr_span.start.byte + 1;
                    let content_end = expr_span.end.byte - 1;

                    // Convert reserved keywords in expressions to safe variable names
                    let safe_expr = self.safe_var_name(expr.trim());

                    if name == "class" {
                        output.push(" ");
                        output.push(name);
                        output.push("=\"{render_class(");
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
                        output.push(")}\"");
                    } else if name == "style" {
                        output.push(" ");
                        output.push(name);
                        output.push("=\"{render_style(");
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
                        output.push(")}\"");
                    } else if self.is_boolean_attribute(name) {
                        // Boolean attrs: entire attribute is conditional
                        output.push("{render_attr(\"");
                        output.push(name);
                        output.push("\", ");
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
                        output.push(")}");
                    } else {
                        output.push(" ");
                        output.push(name);
                        output.push("=\"{escape(");
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
                        output.push(")}\"");
                    }
                }
            }
            AttributeKind::Boolean { name } => {
                output.push(" ");
                output.push(name);
            }
            AttributeKind::Shorthand { name, expr_span } => {
                if in_fstring {
                    // Use safe variable name for reserved keywords
                    let var_name = self.safe_var_name(name);
                    // Shorthand expr_span.end points TO closing brace (not past it),
                    // so content_end = end.byte gives exclusive end of the name content
                    let content_start = expr_span.start.byte + 1;
                    let content_end = expr_span.end.byte;

                    let (start, end) = if name == "class" {
                        output.push(" ");
                        output.push(name);
                        output.push("=\"{render_class(");
                        let s = output.position();
                        output.push(&var_name);
                        let e = output.position();
                        output.push(")}\"");
                        (s, e)
                    } else if name == "style" {
                        output.push(" ");
                        output.push(name);
                        output.push("=\"{render_style(");
                        let s = output.position();
                        output.push(&var_name);
                        let e = output.position();
                        output.push(")}\"");
                        (s, e)
                    } else if name == "data" {
                        output.push("{render_data(");
                        let s = output.position();
                        output.push(&var_name);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    } else if name == "aria" {
                        output.push("{render_aria(");
                        let s = output.position();
                        output.push(&var_name);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    } else if self.is_boolean_attribute(name) {
                        output.push("{render_attr(\"");
                        output.push(name);
                        output.push("\", ");
                        let s = output.position();
                        output.push(&var_name);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    } else {
                        // Generic attribute shorthand - treat as spread
                        output.push("{spread_attrs(");
                        let s = output.position();
                        output.push(&var_name);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    };
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
            AttributeKind::Spread { expr, expr_span } => {
                if in_fstring {
                    // Detect special spread types by variable name
                    // Convert reserved keywords to safe variable names
                    let trimmed_expr = expr.trim();
                    let safe_expr = self.safe_var_name(trimmed_expr);
                    // Spread expr_span.end points TO closing brace (not past it).
                    // Spread syntax is {**expr}, so skip 3 chars for "{**"
                    let content_start = expr_span.start.byte + 3;
                    let content_end = expr_span.end.byte;

                    let (start, end) = if trimmed_expr == "class" {
                        output.push(" class=\"{render_class(");
                        let s = output.position();
                        output.push(&safe_expr);
                        let e = output.position();
                        output.push(")}\"");
                        (s, e)
                    } else if trimmed_expr == "style" {
                        output.push(" style=\"{render_style(");
                        let s = output.position();
                        output.push(&safe_expr);
                        let e = output.position();
                        output.push(")}\"");
                        (s, e)
                    } else if trimmed_expr == "data" {
                        output.push("{render_data(");
                        let s = output.position();
                        output.push(&safe_expr);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    } else if trimmed_expr == "aria" {
                        output.push("{render_aria(");
                        let s = output.position();
                        output.push(&safe_expr);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    } else {
                        // Generic spread
                        output.push("{spread_attrs(");
                        let s = output.position();
                        output.push(&safe_expr);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    };
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
            AttributeKind::SlotAssignment { name, expr, expr_span } => {
                if let Some(e) = expr {
                    if in_fstring {
                        output.push(" slot:");
                        output.push(name);
                        output.push("=\"{");
                        let start = output.position();
                        output.push(e);
                        let end = output.position();
                        output.push("}\"");
                        if let Some(span) = expr_span {
                            // SlotAssignment expr_span.end points TO closing brace
                            let content_start = span.start.byte + 1;
                            let content_end = span.end.byte;
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
                    // Emit template value with position tracking for each {expr}
                    // value_start_byte: skip past `name="` in the source
                    let value_start_byte = attr.span.start.byte + name.len() + 2;
                    let mut byte_offset = 0;
                    let mut chars = value.chars().peekable();
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
                            let safe_expr = self.safe_var_name(expr.trim());
                            output.push("{escape(");
                            let start = output.position();
                            output.push(&safe_expr);
                            let end = output.position();
                            output.push(")}");
                            output.add_range(Range {
                                range_type: RangeType::Python,
                                source_start: value_start_byte + expr_byte_start,
                                source_end: value_start_byte + expr_byte_end,
                                compiled_start: start,
                                compiled_end: end,
                                needs_injection: true,
                            });
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
            }
        }
    }

    /// Convert {expr} in template string to {escape(expr)} for f-string output.
    /// Also escapes double quotes in static parts as &quot; for valid HTML attributes.
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
        let has_format_extras = expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
        if has_format_extras {
            // Format spec, conversion, or debug — emit as f-string
            output.push("yield f\"{");
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
            output.push("}\"");
        } else if expr.escape {
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
        // Check if any attribute requires f-string interpolation
        let needs_fstring = el.attributes.iter().any(|attr| {
            matches!(
                attr.kind,
                AttributeKind::Dynamic { .. }
                    | AttributeKind::Template { .. }
                    | AttributeKind::Shorthand { .. }
                    | AttributeKind::Spread { .. }
            )
        });

        self.indent(output, indent);
        if needs_fstring {
            output.push("yield f\"<");
        } else {
            output.push("yield \"<");
        }
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

        // Add HTML injection ranges for this element
        self.add_html_ranges(el, output);
    }

    fn emit_attribute(&self, attr: &Attribute, output: &mut Output) {
        match &attr.kind {
            AttributeKind::Static { name, value } => {
                output.push(" ");
                output.push(name);
                output.push("=\\\"");
                output.push(&escape_string(&escape_html_attr_quotes(value)));
                output.push("\\\"");
            }
            AttributeKind::Dynamic { name, expr, .. } => {
                output.push(" ");
                output.push(name);
                output.push("=\\\"{escape(");
                output.push(expr);
                output.push(")}\\\"");
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
                self.emit_component_attr(attr, output);
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
                self.emit_component_attr(attr, output);
            }

            output.push(")");
            output.newline();
        }
    }

    /// Emit a single attribute as a Python keyword argument in a component call
    fn emit_component_attr(&self, attr: &Attribute, output: &mut Output) {
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
            AttributeKind::Boolean { name } => {
                output.push(name);
                output.push("=True");
            }
            AttributeKind::Shorthand { name, .. } => {
                let var_name = self.safe_var_name(name);
                output.push("**");
                output.push(&var_name);
            }
            AttributeKind::Spread { expr, .. } => {
                output.push("**");
                output.push(expr.trim());
            }
            AttributeKind::Template { name, value } => {
                output.push(name);
                output.push("=f\"");
                output.push(&self.convert_template_expressions(value));
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
        // This matches how shorthand attributes rename {class} → class_, {type} → type_.
        let owned_statement;
        let statement = if stmt.stmt.starts_with("class ") {
            owned_statement = format!("class_{}", &stmt.stmt["class".len()..]);
            &owned_statement
        } else if stmt.stmt.starts_with("class=") {
            owned_statement = format!("class_{}", &stmt.stmt["class".len()..]);
            &owned_statement
        } else if stmt.stmt.starts_with("type ") {
            owned_statement = format!("type_{}", &stmt.stmt["type".len()..]);
            &owned_statement
        } else if stmt.stmt.starts_with("type=") {
            owned_statement = format!("type_{}", &stmt.stmt["type".len()..]);
            &owned_statement
        } else {
            &stmt.stmt
        };

        // Only create injection range for non-renamed statements
        let is_renamed = statement != &stmt.stmt;

        let start = output.position();

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

        let end = output.position();

        if !is_renamed {
            output.add_range(Range {
                range_type: RangeType::Python,
                source_start: stmt.span.start.byte,
                source_end: stmt.span.end.byte,
                compiled_start: start,
                compiled_end: end,
                needs_injection: true,
            });
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

        // Build imports based on what helpers are actually used in the generated code
        let mut hyper_imports = vec!["html"];

        if code.contains("{escape(") {
            hyper_imports.push("escape");
        }
        if code.contains("{render_class(") {
            hyper_imports.push("render_class");
        }
        if code.contains("{render_attr(") {
            hyper_imports.push("render_attr");
        }
        if code.contains("{render_style(") {
            hyper_imports.push("render_style");
        }
        if code.contains("{render_data(") {
            hyper_imports.push("render_data");
        }
        if code.contains("{render_aria(") {
            hyper_imports.push("render_aria");
        }
        if code.contains("{spread_attrs(") {
            hyper_imports.push("spread_attrs");
        }

        // Add other helpers based on metadata
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
            analyzer.analyze(ast, &code, &ast.source, adjusted_ranges)
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
