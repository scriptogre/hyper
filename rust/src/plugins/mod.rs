mod analysis;
mod async_detect;
mod helper_detect;
mod mutable_default_detect;
mod reserved_keyword;
mod slot_detect;
mod spread_kwargs;

pub use analysis::{Analysis, BLESSED_SPREAD_NAMES, Helper};
pub use async_detect::AsyncDetectionPlugin;
pub use helper_detect::HelperDetectionPlugin;
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
/// | Role      | Reads             | Writes to              | Hook            |
/// |-----------|--------------------|-----------------------|-----------------|
/// | Transform | `&mut Node`        | mutates node in place | `enter`/`exit`  |
/// | Analyze   | `&Node`            | `Analysis` fields     | `enter`/`exit`  |
/// | Guard     | `&Node`, analysis  | nothing               | `finalize()`    |
///
/// Transform plugins run first (so analyzers see the final AST), then
/// analyzers (so guards can read their output).
pub trait Plugin {
    /// Walk each node top-down. Return `false` to skip children.
    fn enter(&mut self, _node: &mut Node, _analysis: &mut Analysis) -> bool {
        true
    }

    /// Called after a node's children have been visited.
    fn exit(&mut self, _node: &mut Node, _analysis: &mut Analysis) {}

    /// Called once after all nodes have been visited. Return `Err` to reject.
    fn finalize(&mut self, _analysis: &Analysis) -> Result<(), CompileError> {
        Ok(())
    }
}

/// Runs plugins over the AST in two phases: transform, then analyze.
pub struct PluginRunner {
    transform: Vec<Box<dyn Plugin>>,
    analyze: Vec<Box<dyn Plugin>>,
}

impl PluginRunner {
    pub fn new() -> Self {
        Self {
            transform: Vec::new(),
            analyze: Vec::new(),
        }
    }

    pub fn add_transform<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.transform.push(Box::new(plugin));
        self
    }

    pub fn add_analyze<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.analyze.push(Box::new(plugin));
        self
    }

    pub fn run(&mut self, ast: &mut Ast) -> Result<Analysis, CompileError> {
        // Phase 1: Transform (rewrite AST nodes)
        let mut unused = Analysis::new();
        for plugin in &mut self.transform {
            Self::walk(&mut ast.nodes, plugin.as_mut(), &mut unused);
        }

        // Phase 2: Analyze (collect facts, validate)
        let mut analysis = Analysis::new();
        for plugin in &mut self.analyze {
            Self::walk(&mut ast.nodes, plugin.as_mut(), &mut analysis);
            plugin.finalize(&analysis)?;
        }

        Ok(analysis)
    }

    fn walk(nodes: &mut Vec<Node>, plugin: &mut dyn Plugin, analysis: &mut Analysis) {
        for node in nodes {
            if plugin.enter(node, analysis) {
                match node {
                    Node::Element(el) => {
                        Self::walk(&mut el.children, plugin, analysis);
                    }
                    Node::Component(c) => {
                        Self::walk(&mut c.children, plugin, analysis);
                        for slot in c.slots.values_mut() {
                            Self::walk(slot, plugin, analysis);
                        }
                    }
                    Node::Fragment(f) => {
                        Self::walk(&mut f.children, plugin, analysis);
                    }
                    Node::Slot(s) => {
                        Self::walk(&mut s.fallback, plugin, analysis);
                    }
                    Node::If(if_node) => {
                        Self::walk(&mut if_node.then_branch, plugin, analysis);
                        for (_, _, branch) in &mut if_node.elif_branches {
                            Self::walk(branch, plugin, analysis);
                        }
                        if let Some(else_branch) = &mut if_node.else_branch {
                            Self::walk(else_branch, plugin, analysis);
                        }
                    }
                    Node::For(for_node) => {
                        Self::walk(&mut for_node.body, plugin, analysis);
                    }
                    Node::Match(match_node) => {
                        for case in &mut match_node.cases {
                            Self::walk(&mut case.body, plugin, analysis);
                        }
                    }
                    Node::While(while_node) => {
                        Self::walk(&mut while_node.body, plugin, analysis);
                    }
                    Node::With(with_node) => {
                        Self::walk(&mut with_node.body, plugin, analysis);
                    }
                    Node::Try(try_node) => {
                        Self::walk(&mut try_node.body, plugin, analysis);
                        for except in &mut try_node.except_clauses {
                            Self::walk(&mut except.body, plugin, analysis);
                        }
                        if let Some(else_clause) = &mut try_node.else_clause {
                            Self::walk(else_clause, plugin, analysis);
                        }
                        if let Some(finally_clause) = &mut try_node.finally_clause {
                            Self::walk(finally_clause, plugin, analysis);
                        }
                    }
                    Node::Definition(def) => {
                        Self::walk(&mut def.body, plugin, analysis);
                    }
                    Node::Text(_)
                    | Node::Expression(_)
                    | Node::Comment(_)
                    | Node::Statement(_)
                    | Node::Import(_)
                    | Node::Parameter(_)
                    | Node::Decorator(_) => {}
                }
            }
            plugin.exit(node, analysis);
        }
    }
}

impl Default for PluginRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard plugins in correct phase order.
pub fn standard_plugins() -> PluginRunner {
    PluginRunner::new()
        // Transform
        .add_transform(ReservedKeywordPlugin)
        // Analyze
        .add_analyze(HelperDetectionPlugin)
        .add_analyze(AsyncDetectionPlugin)
        .add_analyze(SlotDetectionPlugin)
        .add_analyze(MutableDefaultDetectionPlugin)
        .add_analyze(SpreadKwargsPlugin::new())
}
