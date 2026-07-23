use super::{Flow, Plugin, walk};
use crate::ast::{Function, Node};
use crate::error::CompileError;

/// Marks the function async when the template awaits anything.
#[derive(Default)]
pub struct Async {
    is_async: bool,
}

impl Plugin for Async {
    fn run(&mut self, function: &mut Function) -> Result<(), CompileError> {
        walk(&mut function.params, self)?;
        walk(&mut function.body, self)?;
        function.is_async = self.is_async;
        Ok(())
    }

    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            Node::Expression(expr) if expr.expr.contains("await ") => self.is_async = true,
            Node::Statement(stmt) if stmt.stmt.contains("await ") => self.is_async = true,
            Node::For(for_node) if for_node.is_async || for_node.iterable.contains("await ") => {
                self.is_async = true
            }
            Node::With(with_node) if with_node.is_async || with_node.items.contains("await ") => {
                self.is_async = true
            }
            Node::If(if_node) if if_node.condition.contains("await ") => self.is_async = true,
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
