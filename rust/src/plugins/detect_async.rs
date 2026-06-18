use super::{Context, Flow, Plugin};
use crate::ast::Node;
use crate::error::CompileError;

/// Detects if the template uses await and should be async
pub struct DetectAsync;

impl Plugin for DetectAsync {
    fn enter(&mut self, node: &mut Node, ctx: &mut Context) -> Result<Flow, CompileError> {
        match node {
            Node::Expression(expr) if expr.expr.contains("await ") => ctx.is_async = true,
            Node::Statement(stmt) if stmt.stmt.contains("await ") => ctx.is_async = true,
            Node::For(for_node) if for_node.is_async || for_node.iterable.contains("await ") => {
                ctx.is_async = true
            }
            Node::With(with_node) if with_node.is_async || with_node.items.contains("await ") => {
                ctx.is_async = true
            }
            Node::If(if_node) if if_node.condition.contains("await ") => ctx.is_async = true,
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
