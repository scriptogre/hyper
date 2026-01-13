mod helper_detect;
mod async_detect;
mod slot_detect;
mod metadata;

pub use helper_detect::HelperDetectionPlugin;
pub use async_detect::AsyncDetectionPlugin;
pub use slot_detect::SlotDetectionPlugin;
pub use metadata::TransformMetadata;

use crate::ast::{Ast, Node};

/// Visitor trait for AST transformations
pub trait Visitor {
    /// Called before visiting children. Return `false` to skip children.
    fn enter(&mut self, _node: &mut Node, _metadata: &mut TransformMetadata) -> bool {
        true
    }

    /// Called after visiting children.
    fn exit(&mut self, _node: &mut Node, _metadata: &mut TransformMetadata) {}
}

/// Transformer that applies a series of plugins to an AST
pub struct Transformer {
    plugins: Vec<Box<dyn Visitor>>,
    pub metadata: TransformMetadata,
}

impl Transformer {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            metadata: TransformMetadata::new(),
        }
    }

    pub fn add<V: Visitor + 'static>(mut self, visitor: V) -> Self {
        self.plugins.push(Box::new(visitor));
        self
    }

    pub fn transform(&mut self, ast: &mut Ast) -> &TransformMetadata {
        // Run all plugins
        for plugin in &mut self.plugins {
            Self::visit_nodes(&mut ast.nodes, plugin.as_mut(), &mut self.metadata);
        }

        &self.metadata
    }

    fn visit_nodes(nodes: &mut Vec<Node>, visitor: &mut dyn Visitor, metadata: &mut TransformMetadata) {
        for node in nodes {
            if visitor.enter(node, metadata) {
                // Visit children based on node type
                match node {
                    Node::Element(el) => {
                        Self::visit_nodes(&mut el.children, visitor, metadata);
                    }
                    Node::Component(c) => {
                        Self::visit_nodes(&mut c.children, visitor, metadata);
                        for slot in c.slots.values_mut() {
                            Self::visit_nodes(slot, visitor, metadata);
                        }
                    }
                    Node::Fragment(f) => {
                        Self::visit_nodes(&mut f.children, visitor, metadata);
                    }
                    Node::Slot(s) => {
                        Self::visit_nodes(&mut s.fallback, visitor, metadata);
                    }
                    Node::If(if_node) => {
                        Self::visit_nodes(&mut if_node.then_branch, visitor, metadata);
                        for (_, _, branch) in &mut if_node.elif_branches {
                            Self::visit_nodes(branch, visitor, metadata);
                        }
                        if let Some(else_branch) = &mut if_node.else_branch {
                            Self::visit_nodes(else_branch, visitor, metadata);
                        }
                    }
                    Node::For(for_node) => {
                        Self::visit_nodes(&mut for_node.body, visitor, metadata);
                    }
                    Node::Match(match_node) => {
                        for case in &mut match_node.cases {
                            Self::visit_nodes(&mut case.body, visitor, metadata);
                        }
                    }
                    Node::While(while_node) => {
                        Self::visit_nodes(&mut while_node.body, visitor, metadata);
                    }
                    Node::With(with_node) => {
                        Self::visit_nodes(&mut with_node.body, visitor, metadata);
                    }
                    Node::Try(try_node) => {
                        Self::visit_nodes(&mut try_node.body, visitor, metadata);
                        for except in &mut try_node.except_clauses {
                            Self::visit_nodes(&mut except.body, visitor, metadata);
                        }
                        if let Some(else_clause) = &mut try_node.else_clause {
                            Self::visit_nodes(else_clause, visitor, metadata);
                        }
                        if let Some(finally_clause) = &mut try_node.finally_clause {
                            Self::visit_nodes(finally_clause, visitor, metadata);
                        }
                    }
                    Node::Definition(def) => {
                        Self::visit_nodes(&mut def.body, visitor, metadata);
                    }
                    // Leaf nodes
                    Node::Text(_)
                    | Node::Expression(_)
                    | Node::Statement(_)
                    | Node::Import(_)
                    | Node::Parameter(_)
                    | Node::Decorator(_) => {}
                }
            }
            visitor.exit(node, metadata);
        }
    }
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a transformer with the standard plugins
pub fn standard_plugins() -> Transformer {
    Transformer::new()
        .add(HelperDetectionPlugin)
        .add(AsyncDetectionPlugin)
        .add(SlotDetectionPlugin)
}
