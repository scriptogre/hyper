use super::{Flow, Plugin};
use crate::ast::{Node, StatementNode};
use crate::error::{CompileError, ErrorKind, ParseError};

pub struct ComponentControlFlow;

impl Plugin for ComponentControlFlow {
    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            // Nested Python functions own their return and yield semantics.
            Node::Definition(_) => return Ok(Flow::SkipChildren),
            Node::Statement(statement) => validate_statement(statement)?,
            _ => {}
        }
        Ok(Flow::Continue)
    }
}

fn validate_statement(statement: &StatementNode) -> Result<(), CompileError> {
    let code = statement.stmt.trim();

    if let Some(rest) = keyword_rest(code, "return") {
        let rest = rest.trim_start();
        if !rest.is_empty() && !rest.starts_with('#') {
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                "Components cannot return a value.",
                statement.range,
            )
            .with_help("Use bare 'return' to stop rendering.")
            .boxed()
            .into());
        }
    }

    if keyword_rest(code, "yield").is_some() {
        return Err(ParseError::new(
            ErrorKind::InvalidSyntax,
            "Components cannot use explicit yield.",
            statement.range,
        )
        .with_help("Write markup directly; Hyper yields rendered output.")
        .boxed()
        .into());
    }

    Ok(())
}

fn keyword_rest<'a>(code: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = code.strip_prefix(keyword)?;
    if rest.is_empty()
        || rest
            .chars()
            .next()
            .is_some_and(|character| character.is_whitespace() || character == '(')
    {
        Some(rest)
    } else {
        None
    }
}
