mod async_detect;
mod helper_detect;
mod metadata;
mod mutable_default_detect;
mod reserved_keyword;
mod slot_detect;
mod spread_kwargs;

pub use async_detect::AsyncDetectionPlugin;
pub use helper_detect::HelperDetectionPlugin;
pub use metadata::{BLESSED_SPREAD_NAMES, Helper, TransformMetadata};
pub use mutable_default_detect::MutableDefaultDetectionPlugin;
pub use reserved_keyword::{ReservedKeywordPlugin, rename_reserved_keywords};
pub use slot_detect::SlotDetectionPlugin;
pub use spread_kwargs::SpreadKwargsPlugin;

use crate::ast::{Ast, Node};
use crate::error::CompileError;

/// A plugin walks every AST node (via `enter`/`exit`) and optionally runs a
/// final check after the walk (via `finalize`).
///
/// Plugins serve one or more of three roles:
///
/// | Role      | Reads             | Writes to                  | Hook          |
/// |-----------|--------------------|-----------------------------|---------------|
/// | Transform | `&mut Node`        | mutates the node in place  | `enter`/`exit`|
/// | Scan      | `&Node`            | `TransformMetadata` fields | `enter`/`exit`|
/// | Guard     | `&Node`, metadata  | nothing                    | `finalize()`  |
///
/// Order matters: transform plugins run first (so scans see the final AST),
/// then scans (so guards can read metadata), then guards.
pub trait Plugin {
    /// Walk each node top-down. Return `false` to skip children.
    fn enter(&mut self, _node: &mut Node, _metadata: &mut TransformMetadata) -> bool {
        true
    }

    /// Called after a node's children have been visited.
    fn exit(&mut self, _node: &mut Node, _metadata: &mut TransformMetadata) {}

    /// Called once after all nodes have been visited. Return `Err` to reject.
    fn finalize(&mut self, _metadata: &TransformMetadata) -> Result<(), CompileError> {
        Ok(())
    }
}

/// Runs an ordered list of plugins over the AST.
pub struct Transformer {
    plugins: Vec<Box<dyn Plugin>>,
    pub metadata: TransformMetadata,
}

impl Transformer {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            metadata: TransformMetadata::new(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn transform(&mut self, ast: &mut Ast) -> Result<&TransformMetadata, CompileError> {
        for plugin in &mut self.plugins {
            Self::walk(&mut ast.nodes, plugin.as_mut(), &mut self.metadata);
            plugin.finalize(&self.metadata)?;
        }

        Ok(&self.metadata)
    }

    fn walk(nodes: &mut Vec<Node>, plugin: &mut dyn Plugin, metadata: &mut TransformMetadata) {
        for node in nodes {
            if plugin.enter(node, metadata) {
                match node {
                    Node::Element(el) => {
                        Self::walk(&mut el.children, plugin, metadata);
                    }
                    Node::Component(c) => {
                        Self::walk(&mut c.children, plugin, metadata);
                        for slot in c.slots.values_mut() {
                            Self::walk(slot, plugin, metadata);
                        }
                    }
                    Node::Fragment(f) => {
                        Self::walk(&mut f.children, plugin, metadata);
                    }
                    Node::Slot(s) => {
                        Self::walk(&mut s.fallback, plugin, metadata);
                    }
                    Node::If(if_node) => {
                        Self::walk(&mut if_node.then_branch, plugin, metadata);
                        for (_, _, branch) in &mut if_node.elif_branches {
                            Self::walk(branch, plugin, metadata);
                        }
                        if let Some(else_branch) = &mut if_node.else_branch {
                            Self::walk(else_branch, plugin, metadata);
                        }
                    }
                    Node::For(for_node) => {
                        Self::walk(&mut for_node.body, plugin, metadata);
                    }
                    Node::Match(match_node) => {
                        for case in &mut match_node.cases {
                            Self::walk(&mut case.body, plugin, metadata);
                        }
                    }
                    Node::While(while_node) => {
                        Self::walk(&mut while_node.body, plugin, metadata);
                    }
                    Node::With(with_node) => {
                        Self::walk(&mut with_node.body, plugin, metadata);
                    }
                    Node::Try(try_node) => {
                        Self::walk(&mut try_node.body, plugin, metadata);
                        for except in &mut try_node.except_clauses {
                            Self::walk(&mut except.body, plugin, metadata);
                        }
                        if let Some(else_clause) = &mut try_node.else_clause {
                            Self::walk(else_clause, plugin, metadata);
                        }
                        if let Some(finally_clause) = &mut try_node.finally_clause {
                            Self::walk(finally_clause, plugin, metadata);
                        }
                    }
                    Node::Definition(def) => {
                        Self::walk(&mut def.body, plugin, metadata);
                    }
                    // Leaf nodes
                    Node::Text(_)
                    | Node::Expression(_)
                    | Node::Comment(_)
                    | Node::Statement(_)
                    | Node::Import(_)
                    | Node::Parameter(_)
                    | Node::Decorator(_) => {}
                }
            }
            plugin.exit(node, metadata);
        }
    }
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard plugin list. Order matters: transforms first, then scans, then guards.
pub fn standard_plugins() -> Transformer {
    Transformer::new()
        .add(ReservedKeywordPlugin) // transform: rename reserved keywords
        .add(HelperDetectionPlugin) // scan: which runtime helpers are needed
        .add(AsyncDetectionPlugin) // scan: is this template async
        .add(SlotDetectionPlugin) // scan: which slots are used
        .add(MutableDefaultDetectionPlugin) // scan: nullable mutable defaults
        .add(SpreadKwargsPlugin::new()) // scan + guard: blessed spread names
}
