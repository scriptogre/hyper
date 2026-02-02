use super::Visitor;
use crate::ast::Node;

/// Detects if the template uses await and should be async
pub struct AsyncDetectionPlugin;

impl Visitor for AsyncDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Expression(expr) => {
                if expr.expr.contains("await ") {
                    metadata.is_async = true;
                }
            }
            Node::Statement(stmt) => {
                if stmt.stmt.contains("await ") {
                    metadata.is_async = true;
                }
            }
            Node::For(for_node) => {
                if for_node.is_async || for_node.iterable.contains("await ") {
                    metadata.is_async = true;
                }
            }
            Node::With(with_node) => {
                if with_node.is_async || with_node.items.contains("await ") {
                    metadata.is_async = true;
                }
            }
            Node::If(if_node) => {
                if if_node.condition.contains("await ") {
                    metadata.is_async = true;
                }
            }
            _ => {}
        }
        true
    }
}
