use super::tokenizer::{Position, Span, Token};
use crate::ast::*;
use crate::error::{ErrorKind, ParseError, ParseResult};
use crate::html;
use std::collections::HashMap;
use std::sync::Arc;

type ComponentChildren = (Vec<Node>, HashMap<String, Vec<Node>>, Option<Span>);

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

    /// Get a span at the current position (for EOF or current token)
    fn current_span(&self) -> Span {
        if let Some(token) = self.peek() {
            token.span()
        } else {
            // EOF span - point to end of source
            let byte = self.source.len();
            let line = self.source.lines().count().saturating_sub(1);
            let col = self.source.lines().last().map(|l| l.len()).unwrap_or(0);
            Span {
                start: Position { byte, line, col },
                end: Position { byte, line, col },
            }
        }
    }

    /// Require an 'end' token to close a block
    fn expect_end(&mut self, block_keyword: &str, open_span: &Span) -> ParseResult<()> {
        if let Some(Token::End { .. }) = self.peek() {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                ErrorKind::UnclosedBlock,
                format!("This '{}' block is never closed.", block_keyword),
                self.current_span(),
            )
            .with_related(*open_span)
            .with_help("Close with 'end'")
            .boxed())
        }
    }

    fn parse_node(&mut self) -> ParseResult<Option<Node>> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = &self.tokens[self.pos];

        match token {
            Token::Newline { span } => {
                // In content area, preserve newlines as text
                if !self.in_header {
                    let node = Node::Text(TextNode {
                        content: "\n".to_string(),
                        span: *span,
                    });
                    self.advance();
                    Ok(Some(node))
                } else {
                    // In header area, skip newlines
                    self.advance();
                    Ok(None)
                }
            }
            Token::Indent { level, span } => {
                // In content area, preserve indentation as whitespace
                if !self.in_header && *level > 0 {
                    let spaces = " ".repeat(*level);
                    let node = Node::Text(TextNode {
                        content: spaces,
                        span: *span,
                    });
                    self.advance();
                    Ok(Some(node))
                } else {
                    // In header area or zero indent, skip
                    self.advance();
                    Ok(None)
                }
            }

            Token::Text { text, span } => {
                let node = Node::Text(TextNode {
                    content: text.clone(),
                    span: *span,
                });
                self.advance();
                Ok(Some(node))
            }

            Token::Expression { code, span } => {
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
                        span: *span,
                        close_span: None,
                    });
                    self.advance();
                    Ok(Some(node))
                } else {
                    let (expr, format_spec, conversion, debug) = Self::parse_expression_parts(code);
                    let node = Node::Expression(ExpressionNode {
                        expr,
                        span: *span,
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
                tag_span,
                attributes,
                self_closing,
                span,
                ..
            } => {
                let element_span = *span;
                let element_tag = tag.clone();
                let element_tag_span = *tag_span;
                let element_attrs = self.convert_attributes(attributes);
                let is_self_closing = *self_closing;

                // Nesting validation: block elements inside <p>, nested interactive elements
                self.check_nesting(&element_tag, &element_span)?;

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
                        element_span,
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
                self.check_duplicate_attributes(&element_attrs, &element_span)?;

                self.advance();

                let (children, close_span) = if is_self_closing {
                    (Vec::new(), None)
                } else {
                    self.parse_until_element_close(&element_tag, &element_span)?
                };

                Ok(Some(Node::Element(ElementNode {
                    tag: element_tag,
                    tag_span: element_tag_span,
                    attributes: element_attrs,
                    children,
                    self_closing: is_self_closing,
                    span: element_span,
                    close_span,
                })))
            }

            Token::HtmlElementClose { .. } => {
                // Unexpected closing tag at top level - skip it
                self.advance();
                Ok(None)
            }

            Token::ComponentOpen {
                name,
                name_span,
                attributes,
                self_closing,
                span,
            } => {
                let component_name = name.clone();
                let component_name_span = *name_span;
                let component_attrs = self.convert_attributes(attributes);
                let component_span = *span;
                let is_self_closing = *self_closing;

                self.check_duplicate_attributes(&component_attrs, &component_span)?;

                self.advance();

                let (children, slots, close_span) = if is_self_closing {
                    (Vec::new(), HashMap::new(), None)
                } else {
                    self.parse_until_component_close(&component_name, &component_span)?
                };

                Ok(Some(Node::Component(ComponentNode {
                    name: component_name,
                    name_span: component_name_span,
                    attributes: component_attrs,
                    children,
                    slots,
                    span: component_span,
                    close_span,
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
                span,
                rest_span,
            } => {
                let keyword = keyword.clone();
                let rest = rest.clone();
                let span = *span;
                let rest_span = *rest_span;
                self.parse_control_flow(&keyword, &rest, &span, &rest_span)
            }

            Token::PythonStatement { code, span } => {
                let code = code.clone();
                let span = *span;

                // If we're in the header and this looks like a parameter, parse it as such
                if self.in_header && self.is_parameter_declaration(&code) {
                    self.parse_parameter(&code, &span)
                } else if self.is_import_statement(&code) {
                    let node = Node::Import(ImportNode { stmt: code, span });
                    self.advance();
                    Ok(Some(node))
                } else {
                    let node = Node::Statement(StatementNode { stmt: code, span });
                    self.advance();
                    Ok(Some(node))
                }
            }

            Token::Decorator { code, span } => {
                let node = Node::Decorator(DecoratorNode {
                    decorator: code.clone(),
                    span: *span,
                });
                self.advance();
                Ok(Some(node))
            }

            Token::Comment { text, span, inline } => {
                let node = Node::Comment(CommentNode {
                    text: text.clone(),
                    span: *span,
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

            Token::SlotOpen { name, span } => {
                let slot_name = name.clone();
                let slot_span = *span;

                self.advance();

                let (fallback, close_span) = if slot_name.is_some() {
                    self.parse_until_slot_close(&slot_name, &slot_span)?
                } else {
                    (Vec::new(), None)
                };

                Ok(Some(Node::Slot(SlotNode {
                    name: slot_name,
                    fallback,
                    span: slot_span,
                    close_span,
                })))
            }

            Token::SlotClose { .. } => {
                // Unexpected closing tag at top level - skip it
                self.advance();
                Ok(None)
            }

            Token::EscapedBrace { brace, span } => {
                // Treat escaped brace as text
                let node = Node::Text(TextNode {
                    content: brace.to_string(),
                    span: *span,
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

            Token::FragmentStart { name: _, span } => {
                let node = Node::Fragment(FragmentNode {
                    children: Vec::new(), // TODO: parse fragment children
                    span: *span,
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
        span: &Span,
        rest_span: &Span,
    ) -> ParseResult<Option<Node>> {
        match keyword {
            "if" => self.parse_if(rest, span, rest_span),
            "for" => self.parse_for(rest, span, rest_span, false),
            "async for" => self.parse_for(rest, span, rest_span, true),
            "while" => self.parse_while(rest, span, rest_span),
            "match" => self.parse_match(rest, span, rest_span),
            "with" => self.parse_with(rest, span, rest_span, false),
            "async with" => self.parse_with(rest, span, rest_span, true),
            "try" => self.parse_try(span),
            "def" | "async def" => self.parse_function(keyword, rest, span),
            "class" => self.parse_class(rest, span),
            _ => Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                format!("'{}' is not a recognized block keyword.", keyword),
                *span,
            )
            .boxed()),
        }
    }

    fn parse_if(
        &mut self,
        condition: &str,
        span: &Span,
        rest_span: &Span,
    ) -> ParseResult<Option<Node>> {
        let condition_span = *rest_span;
        let if_span = *span;

        self.advance();
        let then_branch = self.parse_until_block_end()?;

        let mut elif_branches = Vec::new();
        let mut else_branch = None;

        while let Some(Token::ControlContinuation {
            keyword,
            rest,
            span,
            rest_span,
        }) = self.peek()
        {
            match keyword.as_str() {
                "elif" => {
                    let elif_cond = rest.clone().unwrap_or_default();
                    // Use rest_span if available, fall back to full span
                    let elif_span = rest_span.unwrap_or(*span);
                    self.advance();
                    let elif_body = self.parse_until_block_end()?;
                    elif_branches.push((elif_cond, elif_span, elif_body));
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
        self.expect_end("if", &if_span)?;

        Ok(Some(Node::If(IfNode {
            condition: condition.to_string(),
            condition_span,
            then_branch,
            elif_branches,
            else_branch,
            span: if_span,
        })))
    }

    fn parse_for(
        &mut self,
        rest: &str,
        span: &Span,
        rest_span: &Span,
        is_async: bool,
    ) -> ParseResult<Option<Node>> {
        // Parse "binding in iterable"
        let parts: Vec<&str> = rest.splitn(2, " in ").collect();
        if parts.len() != 2 {
            let keyword = if is_async { "async for" } else { "for" };
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                format!("This doesn't look like a valid {} loop.", keyword),
                *span,
            )
            .with_help(format!("Syntax: {} x in items:", keyword))
            .boxed());
        }

        let binding = parts[0].trim().to_string();
        let iterable = parts[1].trim().to_string();
        // Calculate binding span: from rest_span start to end of binding text
        let binding_span = Span {
            start: rest_span.start,
            end: Position {
                line: rest_span.start.line,
                col: rest_span.start.col + parts[0].len(),
                byte: rest_span.start.byte + parts[0].len(),
            },
        };
        // Calculate iterable span: rest_span start + offset to "in " + "in ".len()
        let binding_and_in_len = parts[0].len() + " in ".len();
        let iterable_span = Span {
            start: Position {
                line: rest_span.start.line,
                col: rest_span.start.col + binding_and_in_len,
                byte: rest_span.start.byte + binding_and_in_len,
            },
            end: rest_span.end,
        };
        let for_span = *span;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        let keyword = if is_async { "async for" } else { "for" };
        self.expect_end(keyword, &for_span)?;

        Ok(Some(Node::For(ForNode {
            binding,
            binding_span,
            iterable,
            iterable_span,
            body,
            is_async,
            span: for_span,
        })))
    }

    fn parse_while(
        &mut self,
        condition: &str,
        span: &Span,
        rest_span: &Span,
    ) -> ParseResult<Option<Node>> {
        let condition_span = *rest_span;
        let while_span = *span;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        self.expect_end("while", &while_span)?;

        Ok(Some(Node::While(WhileNode {
            condition: condition.to_string(),
            condition_span,
            body,
            span: while_span,
        })))
    }

    fn parse_match(
        &mut self,
        expr: &str,
        span: &Span,
        rest_span: &Span,
    ) -> ParseResult<Option<Node>> {
        let expr_span = *rest_span;
        let match_span = *span;

        self.advance();
        let mut cases = Vec::new();

        // Skip newlines and indents before looking for case statements
        self.skip_structural_tokens();

        while let Some(Token::ControlContinuation {
            keyword,
            rest,
            span,
            rest_span,
        }) = self.peek()
        {
            if keyword == "case" {
                let pattern = rest.clone().unwrap_or_default();
                let pattern_span = rest_span.unwrap_or(*span);
                let case_span = *span;
                self.advance();
                let body = self.parse_until_case_end()?;
                cases.push(CaseNode {
                    pattern,
                    pattern_span,
                    body,
                    span: case_span,
                });

                // Skip newlines and indents before next case
                self.skip_structural_tokens();
            } else {
                break;
            }
        }

        // Require 'end' token
        self.expect_end("match", &match_span)?;

        Ok(Some(Node::Match(MatchNode {
            expr: expr.to_string(),
            expr_span,
            cases,
            span: match_span,
        })))
    }

    fn parse_with(
        &mut self,
        items: &str,
        span: &Span,
        rest_span: &Span,
        is_async: bool,
    ) -> ParseResult<Option<Node>> {
        let items_span = *rest_span;
        let with_span = *span;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        let keyword = if is_async { "async with" } else { "with" };
        self.expect_end(keyword, &with_span)?;

        Ok(Some(Node::With(WithNode {
            items: items.to_string(),
            items_span,
            body,
            is_async,
            span: with_span,
        })))
    }

    fn parse_try(&mut self, span: &Span) -> ParseResult<Option<Node>> {
        let try_span = *span;

        self.advance();
        let body = self.parse_until_block_end()?;

        let mut except_clauses = Vec::new();
        let mut else_clause = None;
        let mut finally_clause = None;

        while let Some(Token::ControlContinuation {
            keyword,
            rest,
            span,
            rest_span,
        }) = self.peek()
        {
            match keyword.as_str() {
                "except" => {
                    let exception = rest.clone();
                    let exception_span = rest_span.or_else(|| rest.as_ref().map(|_| *span));
                    let except_span = *span;
                    self.advance();
                    let except_body = self.parse_until_block_end()?;
                    except_clauses.push(ExceptClause {
                        exception,
                        exception_span,
                        body: except_body,
                        span: except_span,
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
        self.expect_end("try", &try_span)?;

        Ok(Some(Node::Try(TryNode {
            body,
            except_clauses,
            else_clause,
            finally_clause,
            span: try_span,
        })))
    }

    fn parse_function(
        &mut self,
        keyword: &str,
        rest: &str,
        span: &Span,
    ) -> ParseResult<Option<Node>> {
        // Strip trailing colon from rest if present (parsing may include it)
        let rest_trimmed = rest.trim_end_matches(':').trim();
        let signature = format!("{} {}:", keyword, rest_trimmed);
        let signature_span = *span;
        let def_span = *span;

        self.advance();
        let body = if self.in_header {
            self.parse_header_block_body(def_span.start.col)?
        } else {
            let body = self.parse_until_block_end()?;
            self.expect_end("def", &def_span)?;
            body
        };

        Ok(Some(Node::Definition(DefinitionNode {
            kind: DefinitionKind::Function,
            signature,
            signature_span,
            body,
            span: def_span,
        })))
    }

    fn parse_class(&mut self, rest: &str, span: &Span) -> ParseResult<Option<Node>> {
        // Strip trailing colon from rest if present (parsing may include it)
        let rest_trimmed = rest.trim_end_matches(':').trim();
        let signature = format!("class {}:", rest_trimmed);
        let signature_span = *span;
        let class_span = *span;

        self.advance();
        let body = if self.in_header {
            self.parse_header_block_body(class_span.start.col)?
        } else {
            let body = self.parse_until_block_end()?;
            self.expect_end("class", &class_span)?;
            body
        };

        Ok(Some(Node::Definition(DefinitionNode {
            kind: DefinitionKind::Class,
            signature,
            signature_span,
            body,
            span: class_span,
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
        open_span: &Span,
    ) -> ParseResult<(Vec<Node>, Option<Span>)> {
        self.element_stack.push(tag.to_string());
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::HtmlElementClose {
                    tag: close_tag,
                    span: close_span,
                    ..
                }) if close_tag == tag => {
                    let close_span = *close_span;
                    self.advance();
                    self.element_stack.pop();
                    return Ok((nodes, Some(close_span)));
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
            self.current_span(),
        )
        .with_related(*open_span)
        .with_help(format!("Close with </{}> or <{} />", tag, tag))
        .boxed())
    }

    fn parse_until_component_close(
        &mut self,
        name: &str,
        open_span: &Span,
    ) -> ParseResult<ComponentChildren> {
        let mut children = Vec::new();
        let slots = HashMap::new(); // TODO: parse slots

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::ComponentClose {
                    name: close_name,
                    span: close_span,
                    ..
                }) if close_name == name => {
                    let close_span = *close_span;
                    self.advance();
                    return Ok((children, slots, Some(close_span)));
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
            self.current_span(),
        )
        .with_related(*open_span)
        .with_help(format!("Close with </{{{}}}> or <{{{}}} />", name, name))
        .boxed())
    }

    fn parse_until_slot_close(
        &mut self,
        name: &Option<String>,
        open_span: &Span,
    ) -> ParseResult<(Vec<Node>, Option<Span>)> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::SlotClose {
                    name: close_name,
                    span: close_span,
                    ..
                }) if close_name == name => {
                    let close_span = *close_span;
                    self.advance();
                    return Ok((nodes, Some(close_span)));
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
            self.current_span(),
        )
        .with_related(*open_span)
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
                    AttributeValue::Expression(code, span) => AttributeKind::Dynamic {
                        name: attr.name.clone(),
                        expr: code.clone(),
                        expr_span: *span,
                    },
                    AttributeValue::Bool => AttributeKind::Boolean {
                        name: attr.name.clone(),
                    },
                    AttributeValue::Shorthand(name, span) => AttributeKind::Shorthand {
                        name: name.clone(),
                        expr_span: *span,
                    },
                    AttributeValue::Spread(code, span) => AttributeKind::Spread {
                        expr: code.clone(),
                        expr_span: *span,
                    },
                    AttributeValue::SlotAssignment(name, span) => AttributeKind::SlotAssignment {
                        name: name.clone(),
                        expr: None,
                        expr_span: Some(*span),
                    },
                };

                Attribute {
                    kind,
                    span: attr.span,
                }
            })
            .collect()
    }

    fn check_nesting(&self, child_tag: &str, child_span: &Span) -> ParseResult<()> {
        if let Some(parent) = self.element_stack.last() {
            // Block elements cannot appear inside <p>
            if html::is_auto_close_element(parent) && html::is_block_element(child_tag) {
                return Err(ParseError::new(
                    ErrorKind::InvalidNesting,
                    format!("<{}> cannot appear inside <{}>.", child_tag, parent),
                    *child_span,
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
                    *child_span,
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
        _element_span: &Span,
    ) -> ParseResult<()> {
        let mut seen = HashMap::new();
        for attr in attrs {
            let name = match &attr.kind {
                AttributeKind::Static { name, .. }
                | AttributeKind::Dynamic { name, .. }
                | AttributeKind::Template { name, .. }
                | AttributeKind::Boolean { name }
                | AttributeKind::Shorthand { name, .. } => Some(name.as_str()),
                AttributeKind::Spread { .. } | AttributeKind::SlotAssignment { .. } => None,
            };
            if let Some(name) = name {
                if let Some(first_span) = seen.get(name) {
                    return Err(ParseError::new(
                        ErrorKind::DuplicateAttribute,
                        format!("\"{}\" is set twice on this element.", name),
                        attr.span,
                    )
                    .with_related(*first_span)
                    .with_related_label("first use")
                    .boxed());
                }
                seen.insert(name, attr.span);
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

        // Handle **kwargs and *args patterns
        if trimmed.starts_with("**") || trimmed.starts_with("*") {
            // Must have a colon for type annotation: **kwargs: dict, *args: tuple
            return trimmed.contains(':');
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

    fn parse_parameter(&mut self, code: &str, span: &Span) -> ParseResult<Option<Node>> {
        // Parse "name: type" or "name: type = default"
        let parts: Vec<&str> = code.splitn(2, ':').collect();
        if parts.len() != 2 {
            // Not a valid parameter, treat as statement
            let node = Node::Statement(StatementNode {
                stmt: code.to_string(),
                span: *span,
            });
            self.advance();
            return Ok(Some(node));
        }

        let name = parts[0].trim().to_string();
        let rest = parts[1].trim();

        // Reject *args - hyper components use keyword-only arguments
        if name.starts_with('*') && !name.starts_with("**") {
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                "Hyper components don't support *args.".to_string(),
                *span,
            )
            .with_help(
                "Hyper components use keyword-only arguments, so *args (which captures \
                positional arguments) doesn't make sense. If you want to accept extra \
                keyword arguments, use **kwargs instead.",
            )
            .boxed());
        }

        // Check if there's a default value
        let (type_hint, default) = if rest.contains('=') {
            let parts: Vec<&str> = rest.splitn(2, '=').collect();
            (
                Some(parts[0].trim().to_string()),
                Some(parts[1].trim().to_string()),
            )
        } else {
            (Some(rest.to_string()), None)
        };

        let node = Node::Parameter(ParameterNode {
            name,
            type_hint,
            default,
            span: *span,
        });

        self.advance();
        Ok(Some(node))
    }
}
