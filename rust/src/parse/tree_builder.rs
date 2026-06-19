use super::tokenizer::{Position, TextRange, Token};
use crate::ast::*;
use crate::error::{ErrorKind, ParseError, ParseResult};
use crate::html;
use std::collections::HashMap;
use std::sync::Arc;

type ComponentChildren = (Vec<Node>, HashMap<String, Vec<Node>>, Option<TextRange>);

/// Builds an AST from a token stream
pub struct TreeBuilder {
    tokens: Vec<Token>,
    pos: usize,
    source: Arc<str>,
    in_header: bool,            // Track if we're before the --- separator
    element_stack: Vec<String>, // Parent element names for nesting validation
}

impl TreeBuilder {
    pub fn new(tokens: Vec<Token>, source: Arc<str>) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
            element_stack: Vec::new(),
            in_header: true, // Start in header zone
        }
    }

    pub fn build(&mut self) -> ParseResult<Vec<Node>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            if let Some(node) = self.parse_node()? {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    /// Get a range at the current position (for EOF or current token)
    fn current_range(&self) -> TextRange {
        if let Some(token) = self.peek() {
            token.range()
        } else {
            // EOF range - point to end of source
            let byte = self.source.len();
            let line = self.source.lines().count().saturating_sub(1);
            let col = self.source.lines().last().map(|l| l.len()).unwrap_or(0);
            TextRange {
                start: Position { byte, line, col },
                end: Position { byte, line, col },
            }
        }
    }

    /// Require an 'end' token to close a block
    fn expect_end(&mut self, block_keyword: &str, open_range: &TextRange) -> ParseResult<()> {
        if let Some(Token::End { .. }) = self.peek() {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                ErrorKind::UnclosedBlock,
                format!("This '{}' block is never closed.", block_keyword),
                self.current_range(),
            )
            .with_related(*open_range)
            .with_help("Close with 'end'")
            .boxed())
        }
    }

    /// A Newline should become Text("\n") only when it represents content whitespace.
    /// The first Newline after a non-content token (comment, statement, etc.) is just
    /// a line ending — skip it. Subsequent consecutive Newlines are blank lines — keep them.
    /// Any Newline after a content token (text, element, expression) is always content.
    fn newline_is_content(&self) -> bool {
        if self.pos == 0 {
            return false;
        }
        match &self.tokens[self.pos - 1] {
            // After content → always preserve
            Token::Text { .. }
            | Token::Expression { .. }
            | Token::HtmlElementOpen { .. }
            | Token::HtmlElementClose { .. }
            | Token::ComponentOpen { .. }
            | Token::ComponentClose { .. }
            | Token::SlotOpen { .. }
            | Token::SlotClose { .. } => true,
            // After another Newline → blank line, preserve
            Token::Newline { .. } => {
                // But only if we already emitted the previous newline as content.
                // Walk back to find the last non-Newline/non-Indent token.
                // If it was content, all subsequent newlines are content (blank lines).
                // If it was non-content, the first newline was skipped (line ending),
                // so this second newline is the first blank line — preserve it.
                let mut newline_count = 1; // Include the current newline
                let mut i = self.pos;
                while i > 0 {
                    i -= 1;
                    match &self.tokens[i] {
                        Token::Newline { .. } | Token::Indent { .. } => {
                            newline_count += 1;
                        }
                        _ => break,
                    }
                }
                // After non-content: first newline = line ending (skipped),
                // second+ = blank lines (preserve)
                newline_count >= 2
            }
            // After non-content (comment, statement, etc.) → line ending, skip
            _ => false,
        }
    }

    fn parse_node(&mut self) -> ParseResult<Option<Node>> {
        if self.is_at_end() {
            return Ok(None);
        }

        // If we're still in the header zone and encounter a content-producing
        // token without having seen a --- separator, transition to body mode.
        // This ensures newlines and indentation are preserved as content.
        if self.in_header {
            let is_content_token = matches!(
                &self.tokens[self.pos],
                Token::HtmlElementOpen { .. }
                    | Token::Expression { .. }
                    | Token::Text { .. }
                    | Token::ComponentOpen { .. }
                    | Token::SlotOpen { .. }
            );
            if is_content_token {
                self.in_header = false;
            }
        }

        let token = &self.tokens[self.pos];

        match token {
            Token::Newline { range } => {
                if !self.in_header && self.newline_is_content() {
                    let node = Node::Text(TextNode {
                        content: "\n".to_string(),
                        range: *range,
                    });
                    self.advance();
                    Ok(Some(node))
                } else {
                    self.advance();
                    Ok(None)
                }
            }
            Token::Indent { level, range } => {
                // In content area, preserve indentation as whitespace
                if !self.in_header && *level > 0 {
                    let spaces = " ".repeat(*level);
                    let node = Node::Text(TextNode {
                        content: spaces,
                        range: *range,
                    });
                    self.advance();
                    Ok(Some(node))
                } else {
                    // In header area or zero indent, skip
                    self.advance();
                    Ok(None)
                }
            }

            Token::Text { text, range } => {
                let node = Node::Text(TextNode {
                    content: text.clone(),
                    range: *range,
                });
                self.advance();
                Ok(Some(node))
            }

            Token::Expression { code, range } => {
                // Check if this is a slot reference (tokenizer converts {...} to {children})
                // Slot names start with "children" (default slot or named slots like children_sidebar)
                let trimmed = code.trim();
                if trimmed == "children" || trimmed.starts_with("children_") {
                    // Extract slot name: "children" -> None, "children_sidebar" -> Some("sidebar")
                    let slot_name = if trimmed == "children" {
                        None
                    } else {
                        Some(trimmed["children_".len()..].to_string())
                    };

                    let node = Node::Slot(SlotNode {
                        name: slot_name,
                        fallback: Vec::new(),
                        range: *range,
                        close_range: None,
                    });
                    self.advance();
                    Ok(Some(node))
                } else {
                    let (expr, format_spec, conversion, debug) = Self::parse_expression_parts(code);
                    let node = Node::Expression(ExpressionNode {
                        expr,
                        range: *range,
                        escape: true, // Default to escaping
                        format_spec,
                        conversion,
                        debug,
                    });
                    self.advance();
                    Ok(Some(node))
                }
            }

            Token::HtmlElementOpen {
                tag,
                tag_range,
                attributes,
                self_closing,
                range,
                ..
            } => {
                let element_range = *range;
                let element_tag = tag.clone();
                let element_tag_range = *tag_range;
                let element_attrs = self.convert_attributes(attributes);
                let is_self_closing = *self_closing;

                // Nesting validation: block elements inside <p>, nested interactive elements
                self.check_nesting(&element_tag, &element_range)?;

                // Void elements cannot have children or closing tags
                if !is_self_closing && html::is_void_element(&element_tag) {
                    let examples: Vec<&str> = ["br", "img", "input", "hr", "meta"]
                        .iter()
                        .copied()
                        .filter(|e| *e != element_tag.as_str())
                        .take(3)
                        .collect();
                    return Err(ParseError::new(
                        ErrorKind::VoidElementWithContent,
                        format!("<{}> cannot have content or a closing tag.", element_tag),
                        element_range,
                    )
                    .with_help(format!(
                        "<{}> is a void element (like {}). Write it as <{} /> instead.",
                        element_tag,
                        examples
                            .iter()
                            .map(|e| format!("<{}>", e))
                            .collect::<Vec<_>>()
                            .join(", "),
                        element_tag
                    ))
                    .boxed());
                }

                // Check for duplicate attributes
                self.check_duplicate_attributes(&element_attrs, &element_range)?;

                self.advance();

                let (children, close_range) = if is_self_closing {
                    (Vec::new(), None)
                } else {
                    self.parse_until_element_close(&element_tag, &element_range)?
                };

                Ok(Some(Node::Element(ElementNode {
                    tag: element_tag,
                    tag_range: element_tag_range,
                    attributes: element_attrs,
                    children,
                    self_closing: is_self_closing,
                    range: element_range,
                    close_range,
                })))
            }

            Token::HtmlElementClose { .. } => {
                // Unexpected closing tag at top level - skip it
                self.advance();
                Ok(None)
            }

            Token::ComponentOpen {
                name,
                name_range,
                attributes,
                self_closing,
                range,
            } => {
                let component_name = name.clone();
                let component_name_range = *name_range;
                let component_attrs = self.convert_attributes(attributes);
                let component_range = *range;
                let is_self_closing = *self_closing;

                self.check_duplicate_attributes(&component_attrs, &component_range)?;

                self.advance();

                let (children, slots, close_range) = if is_self_closing {
                    (Vec::new(), HashMap::new(), None)
                } else {
                    self.parse_until_component_close(&component_name, &component_range)?
                };

                Ok(Some(Node::Component(ComponentNode {
                    name: component_name,
                    name_range: component_name_range,
                    attributes: component_attrs,
                    children,
                    slots,
                    range: component_range,
                    close_range,
                })))
            }

            Token::ComponentClose { .. } => {
                // Unexpected closing tag at top level - skip it
                self.advance();
                Ok(None)
            }

            Token::ControlStart {
                keyword,
                rest,
                range,
                rest_range,
            } => {
                let keyword = keyword.clone();
                let rest = rest.clone();
                let range = *range;
                let rest_range = *rest_range;
                self.parse_control_flow(&keyword, &rest, &range, &rest_range)
            }

            Token::PythonStatement { code, range } => {
                let code = code.clone();
                let range = *range;

                // If we're in the header and this looks like a parameter, parse it as such
                if self.in_header && self.is_parameter_declaration(&code) {
                    self.parse_parameter(&code, &range)
                } else if self.is_import_statement(&code) {
                    let node = Node::Import(ImportNode { stmt: code, range });
                    self.advance();
                    Ok(Some(node))
                } else {
                    let node = Node::Statement(StatementNode { stmt: code, range });
                    self.advance();
                    Ok(Some(node))
                }
            }

            Token::Decorator { code, range } => {
                let node = Node::Decorator(DecoratorNode {
                    decorator: code.clone(),
                    range: *range,
                });
                self.advance();
                Ok(Some(node))
            }

            Token::Comment {
                text,
                range,
                inline,
            } => {
                let node = Node::Comment(CommentNode {
                    text: text.clone(),
                    range: *range,
                    inline: *inline,
                });
                self.advance();
                Ok(Some(node))
            }

            Token::Separator { .. } => {
                // Mark that we're now in the body zone
                self.in_header = false;
                self.advance();
                Ok(None)
            }

            Token::SlotOpen { name, range } => {
                let slot_name = name.clone();
                let slot_range = *range;

                self.advance();

                let (fallback, close_range) = if slot_name.is_some() {
                    self.parse_until_slot_close(&slot_name, &slot_range)?
                } else {
                    (Vec::new(), None)
                };

                Ok(Some(Node::Slot(SlotNode {
                    name: slot_name,
                    fallback,
                    range: slot_range,
                    close_range,
                })))
            }

            Token::SlotClose { .. } => {
                // Unexpected closing tag at top level - skip it
                self.advance();
                Ok(None)
            }

            Token::EscapedBrace { brace, range } => {
                // Treat escaped brace as text
                let node = Node::Text(TextNode {
                    content: brace.to_string(),
                    range: *range,
                });
                self.advance();
                Ok(Some(node))
            }

            Token::Eof { .. } => {
                // End of file - handled by is_at_end()
                Ok(None)
            }

            Token::End { .. } | Token::ControlContinuation { .. } => {
                // Unexpected at top level - skip and continue
                self.advance();
                Ok(None)
            }

            Token::FragmentStart { name: _, range } => {
                let node = Node::Fragment(FragmentNode {
                    children: Vec::new(), // TODO: parse fragment children
                    range: *range,
                });
                self.advance();
                Ok(Some(node))
            }
        }
    }

    fn parse_control_flow(
        &mut self,
        keyword: &str,
        rest: &str,
        range: &TextRange,
        rest_range: &TextRange,
    ) -> ParseResult<Option<Node>> {
        match keyword {
            "if" => self.parse_if(rest, range, rest_range),
            "for" => self.parse_for(rest, range, rest_range, false),
            "async for" => self.parse_for(rest, range, rest_range, true),
            "while" => self.parse_while(rest, range, rest_range),
            "match" => self.parse_match(rest, range, rest_range),
            "with" => self.parse_with(rest, range, rest_range, false),
            "async with" => self.parse_with(rest, range, rest_range, true),
            "try" => self.parse_try(range),
            "def" | "async def" => self.parse_function(keyword, rest, range),
            "class" => self.parse_class(rest, range),
            _ => Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                format!("'{}' is not a recognized block keyword.", keyword),
                *range,
            )
            .boxed()),
        }
    }

    fn parse_if(
        &mut self,
        condition: &str,
        range: &TextRange,
        rest_range: &TextRange,
    ) -> ParseResult<Option<Node>> {
        let condition_range = *rest_range;
        let if_range = *range;

        self.advance();
        let then_branch = self.parse_until_block_end()?;

        let mut elif_branches = Vec::new();
        let mut else_branch = None;

        while let Some(Token::ControlContinuation {
            keyword,
            rest,
            range,
            rest_range,
        }) = self.peek()
        {
            match keyword.as_str() {
                "elif" => {
                    let elif_cond = rest.clone().unwrap_or_default();
                    // Use rest_range if available, fall back to full range
                    let elif_range = rest_range.unwrap_or(*range);
                    self.advance();
                    let elif_body = self.parse_until_block_end()?;
                    elif_branches.push((elif_cond, elif_range, elif_body));
                }
                "else" => {
                    self.advance();
                    else_branch = Some(self.parse_until_block_end()?);
                    break;
                }
                _ => break,
            }
        }

        // Require 'end' token
        self.expect_end("if", &if_range)?;

        Ok(Some(Node::If(IfNode {
            condition: condition.to_string(),
            condition_range,
            then_branch,
            elif_branches,
            else_branch,
            range: if_range,
        })))
    }

    fn parse_for(
        &mut self,
        rest: &str,
        range: &TextRange,
        rest_range: &TextRange,
        is_async: bool,
    ) -> ParseResult<Option<Node>> {
        // Parse "binding in iterable"
        let parts: Vec<&str> = rest.splitn(2, " in ").collect();
        if parts.len() != 2 {
            let keyword = if is_async { "async for" } else { "for" };
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                format!("This doesn't look like a valid {} loop.", keyword),
                *range,
            )
            .with_help(format!("Syntax: {} x in items:", keyword))
            .boxed());
        }

        let binding = parts[0].trim().to_string();
        let iterable = parts[1].trim().to_string();
        // Calculate binding range: from rest_range start to end of binding text
        let binding_range = TextRange {
            start: rest_range.start,
            end: Position {
                line: rest_range.start.line,
                col: rest_range.start.col + parts[0].len(),
                byte: rest_range.start.byte + parts[0].len(),
            },
        };
        // Calculate iterable range: rest_range start + offset to "in " + "in ".len()
        let binding_and_in_len = parts[0].len() + " in ".len();
        let iterable_range = TextRange {
            start: Position {
                line: rest_range.start.line,
                col: rest_range.start.col + binding_and_in_len,
                byte: rest_range.start.byte + binding_and_in_len,
            },
            end: rest_range.end,
        };
        let for_range = *range;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        let keyword = if is_async { "async for" } else { "for" };
        self.expect_end(keyword, &for_range)?;

        Ok(Some(Node::For(ForNode {
            binding,
            binding_range,
            iterable,
            iterable_range,
            body,
            is_async,
            range: for_range,
        })))
    }

    fn parse_while(
        &mut self,
        condition: &str,
        range: &TextRange,
        rest_range: &TextRange,
    ) -> ParseResult<Option<Node>> {
        let condition_range = *rest_range;
        let while_range = *range;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        self.expect_end("while", &while_range)?;

        Ok(Some(Node::While(WhileNode {
            condition: condition.to_string(),
            condition_range,
            body,
            range: while_range,
        })))
    }

    fn parse_match(
        &mut self,
        expr: &str,
        range: &TextRange,
        rest_range: &TextRange,
    ) -> ParseResult<Option<Node>> {
        let expr_range = *rest_range;
        let match_range = *range;

        self.advance();
        let mut cases = Vec::new();

        // Skip newlines and indents before looking for case statements
        self.skip_structural_tokens();

        while let Some(Token::ControlContinuation {
            keyword,
            rest,
            range,
            rest_range,
        }) = self.peek()
        {
            if keyword == "case" {
                let pattern = rest.clone().unwrap_or_default();
                let pattern_range = rest_range.unwrap_or(*range);
                let case_range = *range;
                self.advance();
                let body = self.parse_until_case_end()?;
                cases.push(CaseNode {
                    pattern,
                    pattern_range,
                    body,
                    range: case_range,
                });

                // Skip newlines and indents before next case
                self.skip_structural_tokens();
            } else {
                break;
            }
        }

        // Require 'end' token
        self.expect_end("match", &match_range)?;

        Ok(Some(Node::Match(MatchNode {
            expr: expr.to_string(),
            expr_range,
            cases,
            range: match_range,
        })))
    }

    fn parse_with(
        &mut self,
        items: &str,
        range: &TextRange,
        rest_range: &TextRange,
        is_async: bool,
    ) -> ParseResult<Option<Node>> {
        let items_range = *rest_range;
        let with_range = *range;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        let keyword = if is_async { "async with" } else { "with" };
        self.expect_end(keyword, &with_range)?;

        Ok(Some(Node::With(WithNode {
            items: items.to_string(),
            items_range,
            body,
            is_async,
            range: with_range,
        })))
    }

    fn parse_try(&mut self, range: &TextRange) -> ParseResult<Option<Node>> {
        let try_range = *range;

        self.advance();
        let body = self.parse_until_block_end()?;

        let mut except_clauses = Vec::new();
        let mut else_clause = None;
        let mut finally_clause = None;

        while let Some(Token::ControlContinuation {
            keyword,
            rest,
            range,
            rest_range,
        }) = self.peek()
        {
            match keyword.as_str() {
                "except" => {
                    let exception = rest.clone();
                    let exception_range = rest_range.or_else(|| rest.as_ref().map(|_| *range));
                    let except_range = *range;
                    self.advance();
                    let except_body = self.parse_until_block_end()?;
                    except_clauses.push(ExceptClause {
                        exception,
                        exception_range,
                        body: except_body,
                        range: except_range,
                    });
                }
                "else" => {
                    self.advance();
                    else_clause = Some(self.parse_until_block_end()?);
                }
                "finally" => {
                    self.advance();
                    finally_clause = Some(self.parse_until_block_end()?);
                    break;
                }
                _ => break,
            }
        }

        // Require 'end' token
        self.expect_end("try", &try_range)?;

        Ok(Some(Node::Try(TryNode {
            body,
            except_clauses,
            else_clause,
            finally_clause,
            range: try_range,
        })))
    }

    fn parse_function(
        &mut self,
        keyword: &str,
        rest: &str,
        range: &TextRange,
    ) -> ParseResult<Option<Node>> {
        // Strip trailing colon from rest if present (parsing may include it)
        let rest_trimmed = rest.trim_end_matches(':').trim();
        let signature = format!("{} {}:", keyword, rest_trimmed);
        let signature_range = *range;
        let def_range = *range;

        self.advance();
        let body = if self.in_header {
            self.parse_header_block_body(def_range.start.col)?
        } else {
            let body = self.parse_until_block_end()?;
            self.expect_end("def", &def_range)?;
            body
        };

        Ok(Some(Node::Definition(DefinitionNode {
            kind: DefinitionKind::Function,
            signature,
            signature_range,
            body,
            range: def_range,
        })))
    }

    fn parse_class(&mut self, rest: &str, range: &TextRange) -> ParseResult<Option<Node>> {
        // Strip trailing colon from rest if present (parsing may include it)
        let rest_trimmed = rest.trim_end_matches(':').trim();
        let signature = format!("class {}:", rest_trimmed);
        let signature_range = *range;
        let class_range = *range;

        self.advance();
        let body = if self.in_header {
            self.parse_header_block_body(class_range.start.col)?
        } else {
            let body = self.parse_until_block_end()?;
            self.expect_end("class", &class_range)?;
            body
        };

        Ok(Some(Node::Definition(DefinitionNode {
            kind: DefinitionKind::Class,
            signature,
            signature_range,
            body,
            range: class_range,
        })))
    }

    /// Parse a block body in the header zone, ending by dedentation rather than 'end'.
    /// The block ends when we see a non-whitespace token at or before `base_col`,
    /// a separator, or EOF. Explicit 'end' is still accepted for backwards compat.
    fn parse_header_block_body(&mut self, base_col: usize) -> ParseResult<Vec<Node>> {
        let mut nodes = Vec::new();
        // Track whether we've consumed an Indent token for the current line.
        // After a Newline resets this to false, a non-Indent token at the start
        // of a line means column 0 (dedented).
        let mut line_indent_seen = false;
        while !self.is_at_end() {
            match self.peek() {
                // Backwards compat: still accept explicit 'end'
                Some(Token::End { .. }) => {
                    self.advance();
                    break;
                }
                // Stop at separator
                Some(Token::Separator { .. }) => break,
                // Newline: reset line tracking
                Some(Token::Newline { .. }) => {
                    line_indent_seen = false;
                    self.advance();
                }
                // Indent at start of a new line: check if dedented
                Some(Token::Indent { level, .. }) => {
                    if *level <= base_col {
                        break;
                    }
                    line_indent_seen = true;
                    self.advance();
                }
                // Non-whitespace token with no preceding Indent = column 0
                Some(_) if !line_indent_seen => {
                    break;
                }
                // Content token within an indented line
                _ => {
                    if let Some(node) = self.parse_node()? {
                        nodes.push(node);
                    }
                }
            }
        }
        Ok(nodes)
    }

    fn parse_until_block_end(&mut self) -> ParseResult<Vec<Node>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::End { .. }) | Some(Token::ControlContinuation { .. }) => break,
                _ => {
                    if let Some(node) = self.parse_node()? {
                        nodes.push(node);
                    }
                }
            }
        }

        Ok(nodes)
    }

    fn parse_until_case_end(&mut self) -> ParseResult<Vec<Node>> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            let should_break = match self.peek() {
                Some(Token::End { .. }) => true,
                Some(Token::ControlContinuation { keyword, .. }) => keyword == "case",
                _ => false,
            };
            if should_break {
                break;
            }

            if let Some(node) = self.parse_node()? {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    fn parse_until_element_close(
        &mut self,
        tag: &str,
        open_range: &TextRange,
    ) -> ParseResult<(Vec<Node>, Option<TextRange>)> {
        self.element_stack.push(tag.to_string());
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::HtmlElementClose {
                    tag: close_tag,
                    range: close_range,
                    ..
                }) if close_tag == tag => {
                    let close_range = *close_range;
                    self.advance();
                    self.element_stack.pop();
                    return Ok((nodes, Some(close_range)));
                }
                _ => {
                    if let Some(node) = self.parse_node()? {
                        nodes.push(node);
                    }
                }
            }
        }

        self.element_stack.pop();
        Err(ParseError::new(
            ErrorKind::UnclosedElement,
            format!("<{}> is never closed.", tag),
            self.current_range(),
        )
        .with_related(*open_range)
        .with_help(format!("Close with </{}> or <{} />", tag, tag))
        .boxed())
    }

    fn parse_until_component_close(
        &mut self,
        name: &str,
        open_range: &TextRange,
    ) -> ParseResult<ComponentChildren> {
        let mut children = Vec::new();
        let slots = HashMap::new(); // TODO: parse slots

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::ComponentClose {
                    name: close_name,
                    range: close_range,
                    ..
                }) if close_name == name => {
                    let close_range = *close_range;
                    self.advance();
                    return Ok((children, slots, Some(close_range)));
                }
                _ => {
                    if let Some(node) = self.parse_node()? {
                        children.push(node);
                    }
                }
            }
        }

        Err(ParseError::new(
            ErrorKind::UnclosedComponent,
            format!("<{{{}}}> is never closed.", name),
            self.current_range(),
        )
        .with_related(*open_range)
        .with_help(format!("Close with </{{{}}}> or <{{{}}} />", name, name))
        .boxed())
    }

    fn parse_until_slot_close(
        &mut self,
        name: &Option<String>,
        open_range: &TextRange,
    ) -> ParseResult<(Vec<Node>, Option<TextRange>)> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::SlotClose {
                    name: close_name,
                    range: close_range,
                    ..
                }) if close_name == name => {
                    let close_range = *close_range;
                    self.advance();
                    return Ok((nodes, Some(close_range)));
                }
                _ => {
                    if let Some(node) = self.parse_node()? {
                        nodes.push(node);
                    }
                }
            }
        }

        let slot_name = name
            .as_ref()
            .map(|n| format!("...{}", n))
            .unwrap_or_else(|| "...".to_string());
        Err(ParseError::new(
            ErrorKind::UnclosedSlot,
            format!("<{{{}}}> is never closed.", slot_name),
            self.current_range(),
        )
        .with_related(*open_range)
        .with_help(format!("Close with </{{{}}}>", slot_name))
        .boxed())
    }

    fn convert_attributes(&self, token_attrs: &[super::tokenizer::Attribute]) -> Vec<Attribute> {
        token_attrs
            .iter()
            .map(|attr| {
                use super::tokenizer::AttributeValue;

                let kind = match &attr.value {
                    AttributeValue::String(s) => {
                        // Check if the string contains unescaped expressions like {expr}
                        // (ignoring {{ and }} which are escaped braces)
                        let without_escaped = s.replace("{{", "").replace("}}", "");
                        if without_escaped.contains('{') && without_escaped.contains('}') {
                            AttributeKind::Template {
                                name: attr.name.clone(),
                                value: s.clone(),
                            }
                        } else {
                            AttributeKind::Static {
                                name: attr.name.clone(),
                                value: s.clone(),
                            }
                        }
                    }
                    AttributeValue::Expression(code, range) => AttributeKind::Expression {
                        name: attr.name.clone(),
                        expr: code.clone(),
                        expr_range: *range,
                    },
                    AttributeValue::Bool => AttributeKind::Boolean {
                        name: attr.name.clone(),
                    },
                    AttributeValue::Shorthand(name, range) => AttributeKind::Shorthand {
                        name: name.clone(),
                        expr_range: *range,
                    },
                    AttributeValue::Spread(code, range) => AttributeKind::Spread {
                        expr: code.clone(),
                        expr_range: *range,
                    },
                    AttributeValue::SlotAssignment(name, range) => AttributeKind::SlotAssignment {
                        name: name.clone(),
                        expr: None,
                        expr_range: Some(*range),
                    },
                };

                Attribute {
                    kind,
                    range: attr.range,
                }
            })
            .collect()
    }

    fn check_nesting(&self, child_tag: &str, child_range: &TextRange) -> ParseResult<()> {
        if let Some(parent) = self.element_stack.last() {
            // Block elements cannot appear inside <p>
            if html::is_auto_close_element(parent) && html::is_block_element(child_tag) {
                return Err(ParseError::new(
                    ErrorKind::InvalidNesting,
                    format!("<{}> cannot appear inside <{}>.", child_tag, parent),
                    *child_range,
                )
                .with_help(format!(
                    "Browsers silently close <{}> when they encounter <{}>, so this renders\n\
                     as <{0}></{0}><{1}>...</{1}> — probably not what you want.",
                    parent, child_tag
                ))
                .boxed());
            }

            // Interactive elements cannot nest inside themselves
            if html::is_interactive_element(parent) && html::is_interactive_element(child_tag) {
                return Err(ParseError::new(
                    ErrorKind::InvalidNesting,
                    format!("<{}> cannot appear inside <{}>.", child_tag, parent),
                    *child_range,
                )
                .with_help("Nesting clickable elements is invalid HTML and causes unpredictable behavior across browsers.")
                .boxed());
            }
        }
        Ok(())
    }

    fn check_duplicate_attributes(
        &self,
        attrs: &[Attribute],
        _element_range: &TextRange,
    ) -> ParseResult<()> {
        let mut seen = HashMap::new();
        for attr in attrs {
            let name = match &attr.kind {
                AttributeKind::Static { name, .. }
                | AttributeKind::Expression { name, .. }
                | AttributeKind::Template { name, .. }
                | AttributeKind::Boolean { name }
                | AttributeKind::Shorthand { name, .. } => Some(name.as_str()),
                AttributeKind::Spread { .. } | AttributeKind::SlotAssignment { .. } => None,
            };
            if let Some(name) = name {
                if let Some(first_range) = seen.get(name) {
                    return Err(ParseError::new(
                        ErrorKind::DuplicateAttribute,
                        format!("\"{}\" is set twice on this element.", name),
                        attr.range,
                    )
                    .with_related(*first_range)
                    .with_related_label("first use")
                    .boxed());
                }
                seen.insert(name, attr.range);
            }
        }
        Ok(())
    }

    fn peek(&self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            Some(&self.tokens[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.peek(), Some(Token::Eof { .. }))
    }

    /// Skip over newlines and indents
    fn skip_structural_tokens(&mut self) {
        while let Some(token) = self.peek() {
            match token {
                Token::Newline { .. } | Token::Indent { .. } => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// Parse an expression string into (expr, format_spec, conversion, debug).
    /// Handles `{count:03d}`, `{items!r}`, `{value=}` syntax.
    fn parse_expression_parts(code: &str) -> (String, Option<String>, Option<char>, bool) {
        let mut expr = code.to_string();
        let mut format_spec = None;
        let mut conversion = None;
        let mut debug = false;

        // 1. Check for debug format: trailing `=` (but not ==, !=, <=, >=)
        let trimmed = expr.trim_end();
        if trimmed.ends_with('=')
            && !trimmed.ends_with("==")
            && !trimmed.ends_with("!=")
            && !trimmed.ends_with("<=")
            && !trimmed.ends_with(">=")
        {
            debug = true;
            expr = trimmed[..trimmed.len() - 1].to_string();
            // Debug format skips escape and uses Python's = format directly
            return (expr, format_spec, conversion, debug);
        }

        // 2. Check for conversion flag: !r, !s, !a at end (at depth 0)
        //    Must be at the very end of the expression
        let trimmed = expr.trim_end();
        if trimmed.len() >= 2 {
            let last_two = &trimmed[trimmed.len() - 2..];
            if matches!(last_two, "!r" | "!s" | "!a") {
                // Verify the '!' is at depth 0
                let bang_pos = trimmed.len() - 2;
                if Self::depth_at_position(trimmed, bang_pos) == 0 {
                    conversion = Some(trimmed.as_bytes()[trimmed.len() - 1] as char);
                    expr = trimmed[..trimmed.len() - 2].to_string();
                    return (expr, format_spec, conversion, debug);
                }
            }
        }

        // 3. Check for format spec: `:` at depth 0 (scanning from right)
        //    Must not be inside brackets, parens, strings, or dict literals
        if let Some(colon_pos) = Self::find_format_colon(&expr) {
            format_spec = Some(expr[colon_pos + 1..].trim_end().to_string());
            expr = expr[..colon_pos].to_string();
        }

        (expr, format_spec, conversion, debug)
    }

    /// Calculate nesting depth at a given byte position in an expression.
    fn depth_at_position(expr: &str, target: usize) -> usize {
        let mut depth: usize = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let bytes = expr.as_bytes();
        let mut i = 0;

        while i < target && i < bytes.len() {
            let ch = bytes[i] as char;
            if in_string {
                if ch == '\\' {
                    i += 1; // skip escaped char
                } else if ch == string_char {
                    in_string = false;
                }
            } else {
                match ch {
                    '"' | '\'' => {
                        in_string = true;
                        string_char = ch;
                    }
                    '(' | '[' | '{' => {
                        depth += 1;
                    }
                    ')' | ']' | '}' => {
                        depth = depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }
            i += 1;
        }
        depth
    }

    /// Find the position of the format spec colon (`:` at depth 0).
    /// Scans from right to left to find the last `:` at depth 0.
    /// Returns None if no format spec colon is found.
    fn find_format_colon(expr: &str) -> Option<usize> {
        // Scan left-to-right tracking depth, record the last `:` at depth 0
        let mut depth: usize = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut last_colon_at_depth_0 = None;
        let bytes = expr.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            let ch = bytes[i] as char;
            if in_string {
                if ch == '\\' {
                    i += 2; // skip escaped char
                    continue;
                }
                if ch == string_char {
                    in_string = false;
                }
            } else {
                match ch {
                    '"' | '\'' => {
                        in_string = true;
                        string_char = ch;
                    }
                    '(' | '[' | '{' => {
                        depth += 1;
                    }
                    ')' | ']' | '}' => {
                        depth = depth.saturating_sub(1);
                    }
                    ':' if depth == 0 => {
                        last_colon_at_depth_0 = Some(i);
                    }
                    _ => {}
                }
            }
            i += 1;
        }

        // Validate: the part after `:` should look like a format spec
        // (not like a dict value or slice). Format specs are typically short
        // and contain format characters like d, f, s, >, <, ^, 0, etc.
        if let Some(pos) = last_colon_at_depth_0 {
            let after = expr[pos + 1..].trim();
            // Reject if empty or if it looks like a dict/ternary (contains spaces with keywords)
            if after.is_empty() {
                return None;
            }
            // Simple heuristic: format specs don't contain unquoted spaces or start with keywords
            // Format specs: "03d", ".2f", ">20", "#x", ",", "+.2f"
            if !after.contains(' ') && !after.contains('\t') {
                return Some(pos);
            }
        }

        None
    }

    fn is_parameter_declaration(&self, code: &str) -> bool {
        let trimmed = code.trim();

        // **kwargs: type annotation optional. *args: requires colon (to reach the error message)
        if trimmed.starts_with("**") {
            return true;
        }
        if trimmed.starts_with('*') && trimmed.contains(':') {
            return true;
        }

        // Simple heuristic: contains ":" before any "=" (to allow defaults)
        // and doesn't contain common statement keywords
        if !code.contains(':') {
            return false;
        }

        if code.starts_with("if ")
            || code.starts_with("for ")
            || code.starts_with("while ")
            || code.starts_with("match ")
            || code.starts_with("with ")
        {
            return false;
        }

        // Check if ":" comes before "=" (parameter with default)
        // or if there's no "=" at all (parameter without default)
        if let Some(colon_pos) = code.find(':') {
            if let Some(equals_pos) = code.find('=') {
                colon_pos < equals_pos
            } else {
                true
            }
        } else {
            false
        }
    }

    fn is_import_statement(&self, code: &str) -> bool {
        let trimmed = code.trim();
        trimmed.starts_with("import ") || trimmed.starts_with("from ")
    }

    fn parse_parameter(&mut self, code: &str, range: &TextRange) -> ParseResult<Option<Node>> {
        // Parse "name: type", "name: type = default", or "**kwargs"
        let parts: Vec<&str> = code.splitn(2, ':').collect();

        let (name, type_hint, default) = if parts.len() == 2 {
            // Has colon: "name: type" or "name: type = default"
            let name = parts[0].trim().to_string();
            let rest = parts[1].trim();

            // Reject *args - hyper components use keyword-only arguments
            if name.starts_with('*') && !name.starts_with("**") {
                return Err(ParseError::new(
                    ErrorKind::InvalidSyntax,
                    "Hyper components don't support *args.".to_string(),
                    *range,
                )
                .with_help(
                    "Hyper components use keyword-only arguments, so *args (which captures \
                    positional arguments) doesn't make sense. If you want to accept extra \
                    keyword arguments, use **kwargs instead.",
                )
                .boxed());
            }

            if rest.contains('=') {
                let eq_parts: Vec<&str> = rest.splitn(2, '=').collect();
                (
                    name,
                    Some(eq_parts[0].trim().to_string()),
                    Some(eq_parts[1].trim().to_string()),
                )
            } else {
                (name, Some(rest.to_string()), None)
            }
        } else if code.trim().starts_with("**") {
            // No colon, but **kwargs — type annotation is optional
            (code.trim().to_string(), None, None)
        } else {
            // Not a valid parameter, treat as statement
            let node = Node::Statement(StatementNode {
                stmt: code.to_string(),
                range: *range,
            });
            self.advance();
            return Ok(Some(node));
        };

        // Hyper components use keyword-only params; **kwargs is the one exception.
        let kind = if name.starts_with("**") {
            ParamKind::VarKeyword
        } else {
            ParamKind::KeywordOnly
        };

        let node = Node::Parameter(ParameterNode {
            name,
            type_hint,
            default,
            kind,
            range: *range,
        });

        self.advance();
        Ok(Some(node))
    }
}
