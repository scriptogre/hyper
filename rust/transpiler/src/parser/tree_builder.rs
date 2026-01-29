use super::tokenizer::{Token, Span, Position};
use crate::ast::*;
use crate::error::{ParseError, ErrorKind};
use crate::html;
use std::collections::HashMap;
use std::sync::Arc;

/// Builds an AST from a token stream
pub struct TreeBuilder {
    tokens: Vec<Token>,
    pos: usize,
    source: Arc<str>,
    in_header: bool, // Track if we're before the --- separator
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

    pub fn build(&mut self) -> Result<Vec<Node>, ParseError> {
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
    fn expect_end(&mut self, block_keyword: &str, open_span: &Span) -> Result<(), ParseError> {
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
            .with_help("Close with 'end'"))
        }
    }

    fn parse_node(&mut self) -> Result<Option<Node>, ParseError> {
        if self.is_at_end() {
            return Ok(None);
        }

        let token = &self.tokens[self.pos];

        match token {
            Token::Newline { .. } | Token::Indent { .. } => {
                // Skip structural tokens at top level
                self.advance();
                Ok(None)
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
                        Some(trimmed.strip_prefix("children_").unwrap().to_string())
                    };

                    let node = Node::Slot(SlotNode {
                        name: slot_name,
                        fallback: Vec::new(),
                        span: *span,
                    });
                    self.advance();
                    Ok(Some(node))
                } else {
                    let node = Node::Expression(ExpressionNode {
                        expr: code.clone(),
                        span: *span,
                        escape: true, // Default to escaping
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
                        examples.iter().map(|e| format!("<{}>", e)).collect::<Vec<_>>().join(", "),
                        element_tag
                    )));
                }

                // Check for duplicate attributes
                self.check_duplicate_attributes(&element_attrs, &element_span)?;

                self.advance();

                let children = if is_self_closing {
                    Vec::new()
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

                let (children, slots) = if is_self_closing {
                    (Vec::new(), HashMap::new())
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
                })))
            }

            Token::ComponentClose { .. } => {
                // Unexpected closing tag at top level - skip it
                self.advance();
                Ok(None)
            }

            Token::ControlStart { keyword, rest, span, rest_span } => {
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
                } else {
                    let node = Node::Statement(StatementNode {
                        stmt: code,
                        span,
                    });
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

            Token::Comment { .. } => {
                // Skip comments
                self.advance();
                Ok(None)
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

                let fallback = if slot_name.is_some() {
                    self.parse_until_slot_close(&slot_name, &slot_span)?
                } else {
                    Vec::new()
                };

                Ok(Some(Node::Slot(SlotNode {
                    name: slot_name,
                    fallback,
                    span: slot_span,
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

            Token::FragmentStart { name, span } => {
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
    ) -> Result<Option<Node>, ParseError> {
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
            )),
        }
    }

    fn parse_if(&mut self, condition: &str, span: &Span, rest_span: &Span) -> Result<Option<Node>, ParseError> {
        let condition_span = *rest_span;
        let if_span = *span;

        self.advance();
        let then_branch = self.parse_until_block_end()?;

        let mut elif_branches = Vec::new();
        let mut else_branch = None;

        while let Some(Token::ControlContinuation { keyword, rest, span, rest_span }) =
            self.peek()
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

    fn parse_for(&mut self, rest: &str, span: &Span, rest_span: &Span, is_async: bool) -> Result<Option<Node>, ParseError> {
        // Parse "binding in iterable"
        let parts: Vec<&str> = rest.splitn(2, " in ").collect();
        if parts.len() != 2 {
            let keyword = if is_async { "async for" } else { "for" };
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                format!("This doesn't look like a valid {} loop.", keyword),
                *span,
            )
            .with_help(format!(
                "Syntax: {} x in items:",
                keyword
            )));
        }

        let binding = parts[0].trim().to_string();
        let iterable = parts[1].trim().to_string();
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
            iterable,
            iterable_span,
            body,
            is_async,
            span: for_span,
        })))
    }

    fn parse_while(&mut self, condition: &str, span: &Span, rest_span: &Span) -> Result<Option<Node>, ParseError> {
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

    fn parse_match(&mut self, expr: &str, span: &Span, rest_span: &Span) -> Result<Option<Node>, ParseError> {
        let expr_span = *rest_span;
        let match_span = *span;

        self.advance();
        let mut cases = Vec::new();

        // Skip newlines and indents before looking for case statements
        self.skip_structural_tokens();

        while let Some(Token::ControlContinuation { keyword, rest, span, rest_span }) = self.peek()
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

    fn parse_with(&mut self, items: &str, span: &Span, rest_span: &Span, is_async: bool) -> Result<Option<Node>, ParseError> {
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

    fn parse_try(&mut self, span: &Span) -> Result<Option<Node>, ParseError> {
        let try_span = *span;

        self.advance();
        let body = self.parse_until_block_end()?;

        let mut except_clauses = Vec::new();
        let mut else_clause = None;
        let mut finally_clause = None;

        while let Some(Token::ControlContinuation { keyword, rest, span, rest_span }) =
            self.peek()
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
    ) -> Result<Option<Node>, ParseError> {
        // Strip trailing colon from rest if present (parsing may include it)
        let rest_trimmed = rest.trim_end_matches(':').trim();
        let signature = format!("{} {}:", keyword, rest_trimmed);
        let signature_span = *span;
        let def_span = *span;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        self.expect_end("def", &def_span)?;

        Ok(Some(Node::Definition(DefinitionNode {
            kind: DefinitionKind::Function,
            signature,
            signature_span,
            body,
            span: def_span,
        })))
    }

    fn parse_class(&mut self, rest: &str, span: &Span) -> Result<Option<Node>, ParseError> {
        // Strip trailing colon from rest if present (parsing may include it)
        let rest_trimmed = rest.trim_end_matches(':').trim();
        let signature = format!("class {}:", rest_trimmed);
        let signature_span = *span;
        let class_span = *span;

        self.advance();
        let body = self.parse_until_block_end()?;

        // Require 'end' token
        self.expect_end("class", &class_span)?;

        Ok(Some(Node::Definition(DefinitionNode {
            kind: DefinitionKind::Class,
            signature,
            signature_span,
            body,
            span: class_span,
        })))
    }

    fn parse_until_block_end(&mut self) -> Result<Vec<Node>, ParseError> {
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

    fn parse_until_case_end(&mut self) -> Result<Vec<Node>, ParseError> {
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

            match self.peek() {
                _ => {
                    if let Some(node) = self.parse_node()? {
                        nodes.push(node);
                    }
                }
            }
        }

        Ok(nodes)
    }

    fn parse_until_element_close(&mut self, tag: &str, open_span: &Span) -> Result<Vec<Node>, ParseError> {
        self.element_stack.push(tag.to_string());
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::HtmlElementClose {
                    tag: close_tag, ..
                }) if close_tag == tag => {
                    self.advance();
                    self.element_stack.pop();
                    return Ok(nodes);
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
        .with_help(format!("Close with </{}> or <{} />", tag, tag)))
    }

    fn parse_until_component_close(
        &mut self,
        name: &str,
        open_span: &Span,
    ) -> Result<(Vec<Node>, HashMap<String, Vec<Node>>), ParseError> {
        let mut children = Vec::new();
        let slots = HashMap::new(); // TODO: parse slots

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::ComponentClose {
                    name: close_name, ..
                }) if close_name == name => {
                    self.advance();
                    return Ok((children, slots));
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
        .with_help(format!("Close with </{{{}}}> or <{{{}}} />", name, name)))
    }

    fn parse_until_slot_close(&mut self, name: &Option<String>, open_span: &Span) -> Result<Vec<Node>, ParseError> {
        let mut nodes = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(Token::SlotClose {
                    name: close_name, ..
                }) if close_name == name => {
                    self.advance();
                    return Ok(nodes);
                }
                _ => {
                    if let Some(node) = self.parse_node()? {
                        nodes.push(node);
                    }
                }
            }
        }

        let slot_name = name.as_ref().map(|n| format!("...{}", n)).unwrap_or_else(|| "...".to_string());
        Err(ParseError::new(
            ErrorKind::UnclosedSlot,
            format!("<{{{}}}> is never closed.", slot_name),
            self.current_span(),
        )
        .with_related(*open_span)
        .with_help(format!("Close with </{{{}}}>", slot_name)))
    }

    fn convert_attributes(&self, token_attrs: &[super::tokenizer::Attribute]) -> Vec<Attribute> {
        token_attrs
            .iter()
            .map(|attr| {
                use super::tokenizer::AttributeValue;

                let kind = match &attr.value {
                    AttributeValue::String(s) => AttributeKind::Static {
                        name: attr.name.clone(),
                        value: s.clone(),
                    },
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
                    AttributeValue::SlotAssignment(name, span) => {
                        AttributeKind::SlotAssignment {
                            name: name.clone(),
                            expr: None,
                            expr_span: Some(*span),
                        }
                    }
                };

                Attribute {
                    kind,
                    span: attr.span,
                }
            })
            .collect()
    }

    fn check_nesting(&self, child_tag: &str, child_span: &Span) -> Result<(), ParseError> {
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
                )));
            }

            // Interactive elements cannot nest inside themselves
            if html::is_interactive_element(parent) && html::is_interactive_element(child_tag) {
                return Err(ParseError::new(
                    ErrorKind::InvalidNesting,
                    format!("<{}> cannot appear inside <{}>.", child_tag, parent),
                    *child_span,
                )
                .with_help("Nesting clickable elements is invalid HTML and causes unpredictable behavior across browsers."));
            }
        }
        Ok(())
    }

    fn check_duplicate_attributes(&self, attrs: &[Attribute], _element_span: &Span) -> Result<(), ParseError> {
        let mut seen = HashMap::new();
        for attr in attrs {
            let name = match &attr.kind {
                AttributeKind::Static { name, .. }
                | AttributeKind::Dynamic { name, .. }
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
                    .with_related_label("first use"));
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
        self.pos >= self.tokens.len()
            || matches!(self.peek(), Some(Token::Eof { .. }))
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

    fn parse_parameter(&mut self, code: &str, span: &Span) -> Result<Option<Node>, ParseError> {
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
