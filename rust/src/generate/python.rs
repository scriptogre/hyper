use super::{
    CompileOptions, CompileResult, Generator, Language, Output, Range,
    collect_component_attr_expr_spans, collect_expression_braces, convert_braces_to_utf16,
    html_ranges_for_component, html_ranges_for_element,
};
use crate::ast::*;
use crate::plugins::{DEFAULT_SLOT_PARAM, Helper, rename_reserved_keywords, slot_param_name};

pub struct PythonGenerator;

impl PythonGenerator {
    pub fn new() -> Self {
        Self
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
                        AttributeKind::Static { .. } | AttributeKind::Boolean { .. }
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
    ///   Phase 1 — Emit to a temp buffer for content analysis (ranges discarded).
    ///   Phase 2 — Emit to real output with skip/dedent active (ranges correct by construction).
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
        // Ranges from this pass are discarded.
        let mut temp = Output::new();
        for node in nodes {
            self.emit_node_content(node, &mut temp, has_expressions);
        }
        let (content, _, _) = temp.finish();
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
        // Ranges recorded by emit_node_content are correct because
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
                let has_format_extras =
                    expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
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
                let content_start = expr.range.start.byte + 1; // skip '{'
                let content_end = expr.range.end.byte - 1; // skip '}'

                output.add_range(Range {
                    range_type: Language::Python,
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
        for range in html_ranges_for_element(el) {
            output.add_range(range);
        }
    }

    fn is_boolean_attribute(&self, name: &str) -> bool {
        crate::html::is_boolean_attribute(name)
    }

    /// Emit attribute content as part of a string literal
    fn emit_element_attribute(&self, attr: &Attribute, output: &mut Output, in_fstring: bool) {
        match &attr.kind {
            AttributeKind::Static { name, value } => {
                output.push(" ");
                output.push(name);
                output.push("=\"");
                output.push(&escape_html_attr_quotes(value));
                output.push("\"");
            }
            AttributeKind::Expression {
                name,
                expr,
                expr_range,
            } => {
                if in_fstring {
                    // expr_range includes {expr}, skip braces for injection
                    let content_start = expr_range.start.byte + 1;
                    let content_end = expr_range.end.byte - 1;

                    // Already renamed in the AST by ReservedKeywordPlugin.
                    let safe_expr = expr.trim().to_string();

                    if name == "class" {
                        output.push(" ");
                        output.push(name);
                        output.push("=\"{render_class(");
                        let start = output.position();
                        output.push(&safe_expr);
                        let end = output.position();
                        output.add_range(Range {
                            range_type: Language::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                            html_prefix: None,
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
                            range_type: Language::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                            html_prefix: None,
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
                            range_type: Language::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                            html_prefix: None,
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
                            range_type: Language::Python,
                            source_start: content_start,
                            source_end: content_end,
                            compiled_start: start,
                            compiled_end: end,
                            needs_injection: true,
                            html_prefix: None,
                        });
                        output.push(")}\"");
                    }
                }
            }
            AttributeKind::Boolean { name } => {
                output.push(" ");
                output.push(name);
            }
            AttributeKind::Shorthand { name, expr_range } => {
                if in_fstring {
                    // Shorthand maps one AST field to two outputs: the HTML attr name
                    // stays, the Python value variable renames. So rename here.
                    let var_name = rename_reserved_keywords(name);
                    // Shorthand expr_range.end points TO closing brace (not past it),
                    // so content_end = end.byte gives exclusive end of the name content
                    let content_start = expr_range.start.byte + 1;
                    let content_end = expr_range.end.byte;

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
                    } else {
                        output.push("{render_attr(\"");
                        output.push(name);
                        output.push("\", ");
                        let s = output.position();
                        output.push(&var_name);
                        let e = output.position();
                        output.push(")}");
                        (s, e)
                    };
                    output.add_range(Range {
                        range_type: Language::Python,
                        source_start: content_start,
                        source_end: content_end,
                        compiled_start: start,
                        compiled_end: end,
                        needs_injection: true,
                        html_prefix: None,
                    });
                }
            }
            AttributeKind::Spread { expr, expr_range } => {
                if in_fstring {
                    // Spread expr is already renamed in the AST by ReservedKeywordPlugin.
                    let safe_expr = expr.trim().to_string();
                    // Spread expr_range: {**expr} — skip 3 chars for "{**"
                    let content_start = expr_range.start.byte + 3;
                    let content_end = expr_range.end.byte;

                    output.push("{spread_attrs(");
                    let s = output.position();
                    output.push(&safe_expr);
                    let e = output.position();
                    output.push(")}");
                    output.add_range(Range {
                        range_type: Language::Python,
                        source_start: content_start,
                        source_end: content_end,
                        compiled_start: s,
                        compiled_end: e,
                        needs_injection: true,
                        html_prefix: None,
                    });
                }
            }
            AttributeKind::SlotAssignment {
                name,
                expr,
                expr_range,
            } => {
                if let Some(e) = expr {
                    if in_fstring {
                        output.push(" slot:");
                        output.push(name);
                        output.push("=\"{");
                        let start = output.position();
                        output.push(e);
                        let end = output.position();
                        output.push("}\"");
                        if let Some(range) = expr_range {
                            // SlotAssignment expr_range.end points TO closing brace
                            let content_start = range.start.byte + 1;
                            let content_end = range.end.byte;
                            output.add_range(Range {
                                range_type: Language::Python,
                                source_start: content_start,
                                source_end: content_end,
                                compiled_start: start,
                                compiled_end: end,
                                needs_injection: true,
                                html_prefix: None,
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
                            output.push("{escape(");
                            let start = output.position();
                            output.push(&safe_expr);
                            let end = output.position();
                            output.push(")}");
                            output.add_range(Range {
                                range_type: Language::Python,
                                source_start: value_start_byte + expr_byte_start,
                                source_end: value_start_byte + expr_byte_end,
                                compiled_start: start,
                                compiled_end: end,
                                needs_injection: true,
                                html_prefix: None,
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
        let has_format_extras =
            expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
        if has_format_extras {
            // Format spec, conversion, or debug — emit as f-string
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
            output.add_range(Range {
                range_type: Language::Python,
                source_start: expr.range.start.byte + 1,
                source_end: expr.range.end.byte - 1,
                compiled_start: start,
                compiled_end: end,
                needs_injection: true,
                html_prefix: None,
            });
        } else if expr.escape {
            output.push("yield escape(");
            let start = output.position();
            output.push(&expr.expr);
            let end = output.position();
            output.push(")");
            output.add_range(Range {
                range_type: Language::Python,
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
            output.add_range(Range {
                range_type: Language::Python,
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
            output.push(" />\"\"\"");
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

        // Add HTML injection ranges for this element
        for range in html_ranges_for_element(el) {
            output.add_range(range);
        }
    }

    /// Generate the name of the local buffer function that holds a component
    /// call's default-slot content, e.g. `Inner` -> `_inner_default_slot`.
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
        result.push_str(DEFAULT_SLOT_PARAM);
        result
    }

    fn emit_component(&self, c: &ComponentNode, output: &mut Output, indent: usize) {
        let has_children = !c.children.is_empty();
        let name_compiled_start;
        let name_compiled_end;

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
            name_compiled_start = output.position();
            output.push(&c.name);
            name_compiled_end = output.position();
            output.push("(");
            output.push(&func_name);
            output.push("()");

            // Emit attributes as keyword arguments
            for attr in &c.attributes {
                output.push(", ");
                self.emit_component_attribute(attr, output);
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
            name_compiled_start = output.position();
            output.push(&c.name);
            name_compiled_end = output.position();
            output.push("(");

            // Emit attributes as keyword arguments
            let mut first = true;
            for attr in &c.attributes {
                if !first {
                    output.push(", ");
                }
                first = false;
                self.emit_component_attribute(attr, output);
            }

            output.push(")");
            output.newline();
        }

        // Add Python range for the component name in the opening tag
        // This enables go-to-definition and highlighting for the name
        output.add_range(Range {
            range_type: Language::Python,
            source_start: c.name_range.start.byte,
            source_end: c.name_range.end.byte,
            compiled_start: name_compiled_start,
            compiled_end: name_compiled_end,
            needs_injection: true,
            html_prefix: None,
        });

        // Add Python range for the component name in the closing tag.
        // needs_injection: false — this is for highlighting only, not for
        // building the virtual Python file (which would duplicate the name).
        if let Some(ref cs) = c.close_range {
            // Closing tag is </{Name}> — name starts at byte+3 (skip "</{"), ends at byte-2 (skip "}>")
            let close_name_start = cs.start.byte + 3;
            let close_name_end = cs.end.byte - 2;
            if close_name_end > close_name_start {
                output.add_range(Range {
                    range_type: Language::Python,
                    source_start: close_name_start,
                    source_end: close_name_end,
                    compiled_start: 0,
                    compiled_end: 0,
                    needs_injection: false,
                    html_prefix: None,
                });
            }
        }

        // Add HTML ranges for component tag angle brackets,
        // splitting around attribute expression spans to avoid overlap
        let brace_open = c.name_range.start.byte - 1;
        let brace_close = c.name_range.end.byte;
        let attr_expr_spans = collect_component_attr_expr_spans(&c.attributes);
        for range in html_ranges_for_component(
            &c.range,
            c.close_range.as_ref(),
            brace_open,
            brace_close,
            &attr_expr_spans,
        ) {
            output.add_range(range);
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
                output.add_range(Range {
                    range_type: Language::Python,
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
                output.add_range(Range {
                    range_type: Language::Python,
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
                output.add_range(Range {
                    range_type: Language::Python,
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

        // Add HTML ranges for tag-form slot angle brackets (<{...name}> / </{...name}>)
        // Slots have no attributes, so no expression spans to exclude
        if s.close_range.is_some() {
            let brace_open = s.range.start.byte + 1;
            let brace_close = s.range.end.byte - 2;
            for range in html_ranges_for_component(
                &s.range,
                s.close_range.as_ref(),
                brace_open,
                brace_close,
                &[],
            ) {
                output.add_range(range);
            }
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
        // Track condition for Python injection (skip compiler-generated guards)
        // Adjust source_end to match actual content (trim trailing : and whitespace)
        if !if_node.condition_range.is_synthetic() {
            let source_end = if_node.condition_range.start.byte + condition.len();
            output.add_range(Range {
                range_type: Language::Python,
                source_start: if_node.condition_range.start.byte,
                source_end,
                compiled_start: cond_start,
                compiled_end: cond_end,
                needs_injection: true,
                html_prefix: None,
            });
        }
        output.push(":");
        output.newline();

        self.emit_body_or_pass(&if_node.then_branch, output, indent + 1);

        for (condition, condition_range, body) in &if_node.elif_branches {
            self.indent(output, indent);
            output.push("elif ");
            let condition = condition.trim_end_matches(':').trim();
            let cond_start = output.position();
            output.push(condition);
            let cond_end = output.position();
            if !condition_range.is_synthetic() {
                let source_end = condition_range.start.byte + condition.len();
                output.add_range(Range {
                    range_type: Language::Python,
                    source_start: condition_range.start.byte,
                    source_end,
                    compiled_start: cond_start,
                    compiled_end: cond_end,
                    needs_injection: true,
                    html_prefix: None,
                });
            }
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
        // Remove trailing colon from iterable if present (parsing includes it)
        let iterable = for_node.iterable.trim_end_matches(':').trim();
        let binding_start = output.position();
        output.push(&for_node.binding);
        output.push(" in ");
        output.push(iterable);
        let range_end = output.position();
        let source_end = for_node.iterable_range.start.byte + iterable.len();
        output.add_range(Range {
            range_type: Language::Python,
            source_start: for_node.binding_range.start.byte,
            source_end,
            compiled_start: binding_start,
            compiled_end: range_end,
            needs_injection: true,
            html_prefix: None,
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
        let source_end = match_node.expr_range.start.byte + expr.len();
        output.add_range(Range {
            range_type: Language::Python,
            source_start: match_node.expr_range.start.byte,
            source_end,
            compiled_start: expr_start,
            compiled_end: expr_end,
            needs_injection: true,
            html_prefix: None,
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
            let source_end = case.pattern_range.start.byte + pattern.len();
            output.add_range(Range {
                range_type: Language::Python,
                source_start: case.pattern_range.start.byte,
                source_end,
                compiled_start: pat_start,
                compiled_end: pat_end,
                needs_injection: true,
                html_prefix: None,
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
        let source_end = while_node.condition_range.start.byte + condition.len();
        output.add_range(Range {
            range_type: Language::Python,
            source_start: while_node.condition_range.start.byte,
            source_end,
            compiled_start: cond_start,
            compiled_end: cond_end,
            needs_injection: true,
            html_prefix: None,
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
        let source_end = with_node.items_range.start.byte + items.len();
        output.add_range(Range {
            range_type: Language::Python,
            source_start: with_node.items_range.start.byte,
            source_end,
            compiled_start: items_start,
            compiled_end: items_end,
            needs_injection: true,
            html_prefix: None,
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
                let start = output.position();
                output.push(exception);
                let end = output.position();
                if let Some(ref exc_range) = except.exception_range {
                    let source_end = exc_range.start.byte + exception.len();
                    output.add_range(Range {
                        range_type: Language::Python,
                        source_start: exc_range.start.byte,
                        source_end,
                        compiled_start: start,
                        compiled_end: end,
                        needs_injection: true,
                        html_prefix: None,
                    });
                }
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

        let start = output.position();

        // For multiline statements, add indent to each continuation line
        if stmt.stmt.contains('\n') {
            let indent_str = "    ".repeat(indent);
            let lines: Vec<&str> = stmt.stmt.split('\n').collect();
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
            output.push(&stmt.stmt);
        }

        let end = output.position();

        // Skip injection for compiler-generated statements (no source location).
        if !stmt.range.is_synthetic() {
            output.add_range(Range {
                range_type: Language::Python,
                source_start: stmt.range.start.byte,
                source_end: stmt.range.end.byte,
                compiled_start: start,
                compiled_end: end,
                needs_injection: true,
                html_prefix: None,
            });
        }

        output.newline();
    }

    fn emit_definition(&self, def: &DefinitionNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        let start = output.position();
        output.push(&def.signature);
        let end = output.position();
        output.add_range(Range {
            range_type: Language::Python,
            source_start: def.signature_range.start.byte,
            source_end: def.signature_range.end.byte,
            compiled_start: start,
            compiled_end: end,
            needs_injection: true,
            html_prefix: None,
        });
        output.newline();

        self.emit_body_or_pass(&def.body, output, indent + 1);
    }

    fn emit_import(&self, import: &ImportNode, output: &mut Output, _indent: usize) {
        let start = output.position();
        output.push(&import.stmt);
        let end = output.position();
        output.add_range(Range {
            range_type: Language::Python,
            source_start: import.range.start.byte,
            source_end: import.range.end.byte,
            compiled_start: start,
            compiled_end: end,
            needs_injection: true,
            html_prefix: None,
        });
        output.newline();
    }

    fn emit_decorator(&self, dec: &DecoratorNode, output: &mut Output, indent: usize) {
        self.indent(output, indent);
        let start = output.position();
        output.push(&dec.decorator);
        let end = output.position();
        output.add_range(Range {
            range_type: Language::Python,
            source_start: dec.range.start.byte,
            source_end: dec.range.end.byte,
            compiled_start: start,
            compiled_end: end,
            needs_injection: true,
            html_prefix: None,
        });
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
            output.add_range(Range {
                range_type: Language::Python,
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
}

impl Default for PythonGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for PythonGenerator {
    fn generate(
        &self,
        ast: &Ast,
        ctx: &crate::plugins::Context,
        options: &CompileOptions,
    ) -> CompileResult {
        let mut output = Output::new();

        // Frontmatter and body are already split by the `lower` pass.
        let function = &ast.function;
        let parameters: Vec<&ParameterNode> = function
            .params
            .iter()
            .filter_map(|n| match n {
                Node::Parameter(p) => Some(p),
                _ => None,
            })
            .collect();
        let imports: Vec<&ImportNode> = function.imports.iter().collect();
        let decorators: Vec<&DecoratorNode> = function.decorators.iter().collect();
        let header_comments: Vec<&CommentNode> = function.header_comments.iter().collect();
        let body_nodes: Vec<&Node> = function.body.iter().collect();

        // Emit user imports
        for import in &imports {
            let import_start = output.position();
            output.push(&import.stmt);
            let import_end = output.position();
            output.newline();

            output.add_range(Range {
                range_type: Language::Python,
                source_start: import.range.start.byte,
                source_end: import.range.end.byte,
                compiled_start: import_start,
                compiled_end: import_end,
                needs_injection: true,
                html_prefix: None,
            });
        }

        // Note: orphaned decorators (not attached to inner defs) are applied to
        // the outer template function, emitted later alongside @html in the
        // import block insertion step.

        // Emit function signature with parameters
        let func_name = options
            .function_name
            .as_deref()
            .map(to_pascal_case)
            .unwrap_or_else(|| "Render".to_string());

        // Add async if needed
        if function.is_async {
            output.push("async def ");
        } else {
            output.push("def ");
        }
        output.push(&func_name);

        // Partition params by where they sit in the signature.
        let positional: Vec<&ParameterNode> = parameters
            .iter()
            .copied()
            .filter(|p| p.kind == ParamKind::Positional)
            .collect();
        let keyword_only: Vec<&ParameterNode> = parameters
            .iter()
            .copied()
            .filter(|p| p.kind == ParamKind::KeywordOnly)
            .collect();
        let var_keyword = parameters
            .iter()
            .copied()
            .find(|p| p.kind == ParamKind::VarKeyword);

        let has_any_params =
            !positional.is_empty() || !keyword_only.is_empty() || var_keyword.is_some();

        if !has_any_params {
            output.push("():");
            output.newline();
        } else {
            output.push("(");
            output.newline();
            let sig_indent = "        "; // 8 spaces, double indent for continuation

            // Positional params (before the keyword-only marker): the default slot.
            for param in &positional {
                self.emit_signature_param(param, &mut output, sig_indent);
            }

            // Keyword-only marker, only when keyword-only params follow.
            if !keyword_only.is_empty() {
                output.push(sig_indent);
                output.push("*,");
                output.newline();
            }

            // Keyword-only params: user params, then named slots.
            for param in &keyword_only {
                self.emit_signature_param(param, &mut output, sig_indent);
            }

            // **kwargs (explicit or injected by SpreadKwargs); never an injection.
            if let Some(kwargs) = var_keyword {
                output.push(sig_indent);
                output.push(&kwargs.name);
                if let Some(type_hint) = &kwargs.type_hint {
                    output.push(": ");
                    output.push(type_hint);
                }
                output.push(",");
                output.newline();
            }

            output.push("):");
            output.newline();
        }

        // Emit body. Mutable-default guards are already prepended to the body
        // by DetectMutableDefaults, so an empty body means a genuinely empty one.
        if body_nodes.is_empty() || self.is_effectively_empty(&body_nodes) {
            self.indent(&mut output, 1);
            output.push("pass");
            output.newline();
        } else {
            self.emit_nodes(&body_nodes, &mut output, 1);
        }

        let (mut code, mappings, tracked_ranges) = output.finish();

        // Iterable import is needed when a param is typed with it (slot params).
        let needs_iterable = parameters.iter().any(|p| {
            p.type_hint
                .as_deref()
                .is_some_and(|t| t.contains("Iterable"))
        });

        // Build imports from ctx (populated by HelperDetectionPlugin)
        let mut hyper_imports = vec!["html"];

        for helper in Helper::ALL {
            if ctx.helpers_used.contains(helper) {
                hyper_imports.push(helper.import_name());
            }
        }

        // Detect typing constructs needed from parameter type hints
        let mut typing_imports: Vec<&str> = Vec::new();
        let all_type_hints: String = parameters
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
            import_lines.push_str(&format!(
                "from typing import {}\n",
                typing_imports.join(", ")
            ));
        }

        // Add Iterable import if needed
        if needs_iterable {
            import_lines.push_str("from collections.abc import Iterable\n");
        }

        // Add hyper imports
        import_lines.push_str(&format!("from hyper import {}\n", hyper_imports.join(", ")));
        import_lines.push_str("\n\n"); // Two blank lines before function (PEP 8)

        // Add header comments (above --- separator)
        for comment in &header_comments {
            import_lines.push_str(&comment.text);
            import_lines.push('\n');
        }

        // Add user decorators for the outer template function (before @html)
        for dec in &decorators {
            import_lines.push_str(&dec.decorator);
            import_lines.push('\n');
        }

        // Add @html decorator
        import_lines.push_str("@html\n");

        // Insert imports before function definition
        // Search for "async def" first to avoid matching "def" inside "async def"
        let import_offset =
            if let Some(def_pos) = code.find("async def ").or_else(|| code.find("def ")) {
                code.insert_str(def_pos, &import_lines);
                import_lines.len()
            } else {
                code.insert_str(0, &import_lines);
                import_lines.len()
            };

        // Compute injection ranges and injections using the analyzer (if requested)
        let (ranges, injections, expression_braces, tag_highlights) = if options.include_ranges {
            // Find insertion point (where import_lines were inserted) in pre-insertion coordinates
            let def_pos = code
                .find("async def ")
                .or_else(|| code.find("def "))
                .unwrap_or(0);
            let pre_insertion_def_pos = def_pos - import_offset;

            // Adjust tracked ranges by the import line offset, but only for ranges
            // at or after the insertion point (user imports come before it)
            let adjusted_ranges: Vec<crate::generate::Range> = tracked_ranges
                .into_iter()
                .map(|mut r| {
                    if r.compiled_start >= pre_insertion_def_pos {
                        r.compiled_start += import_offset;
                        r.compiled_end += import_offset;
                    }
                    r
                })
                .collect();

            let analyzer = super::InjectionAnalyzer::new();
            let (ranges, injections) = analyzer.analyze(ast, &code, &ast.source, adjusted_ranges);

            // Collect expression brace positions from the AST
            let byte_braces = collect_expression_braces(ast);
            let expression_braces = convert_braces_to_utf16(&ast.source, &byte_braces);

            // Collect tag highlight positions for component/slot tags
            let byte_tag_highlights = super::collect_tag_highlights(ast);
            let tag_highlights =
                super::convert_tag_highlights_to_utf16(&ast.source, &byte_tag_highlights);

            (ranges, injections, expression_braces, tag_highlights)
        } else {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        };

        CompileResult {
            code,
            mappings,
            ranges,
            injections,
            expression_braces,
            tag_highlights,
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
