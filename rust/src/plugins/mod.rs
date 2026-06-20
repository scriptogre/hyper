mod r#async;
mod context;
mod mutable_defaults;
mod rename_reserved_keywords;
mod slots;
mod spread_kwargs;

pub use r#async::Async;
pub use context::{BLESSED_SPREAD_NAMES, Helper};
pub use mutable_defaults::MutableDefaults;
pub use rename_reserved_keywords::{RenameReservedKeywords, rename_reserved_keywords};
pub use slots::{DEFAULT_SLOT_PARAM, Slots, slot_param_name};
pub use spread_kwargs::SpreadKwargs;

use crate::ast::{Ast, Node};
use crate::error::CompileError;

/// Whether [`walk`] descends into a node's children after `enter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    SkipChildren,
}

/// A compiler plugin. Reads and rewrites the AST.
///
/// Override `enter`/`exit` for per-node work (the common case). Override `run`
/// to own the traversal: walk twice, reorder nodes via `&mut Ast`, or guard
/// after the walk. Local state lives on the plugin struct.
pub trait Plugin {
    /// Run the plugin over the whole tree. Default walks top-down, calling
    /// `enter` then `exit` on each node.
    fn run(&mut self, ast: &mut Ast) -> Result<(), CompileError> {
        walk(&mut ast.function.params, self)?;
        walk(&mut ast.function.body, self)
    }

    /// Called before a node's children. Return [`Flow::SkipChildren`] to prune.
    fn enter(&mut self, _node: &mut Node) -> Result<Flow, CompileError> {
        Ok(Flow::Continue)
    }

    /// Called after a node's children.
    fn exit(&mut self, _node: &mut Node) -> Result<(), CompileError> {
        Ok(())
    }
}

/// Recurse the tree, calling `plugin.enter` (then `exit`) on each node. The one
/// place that knows the AST shape; plugins reuse it instead of reimplementing it.
pub fn walk<P: Plugin + ?Sized>(nodes: &mut [Node], plugin: &mut P) -> Result<(), CompileError> {
    for node in nodes {
        if plugin.enter(node)? == Flow::Continue {
            match node {
                Node::Element(el) => walk(&mut el.children, plugin)?,
                Node::Component(c) => {
                    walk(&mut c.children, plugin)?;
                    for slot in c.slots.values_mut() {
                        walk(slot, plugin)?;
                    }
                }
                Node::Fragment(f) => walk(&mut f.children, plugin)?,
                Node::Slot(s) => walk(&mut s.fallback, plugin)?,
                Node::If(if_node) => {
                    walk(&mut if_node.then_branch, plugin)?;
                    for (_, _, branch) in &mut if_node.elif_branches {
                        walk(branch, plugin)?;
                    }
                    if let Some(else_branch) = &mut if_node.else_branch {
                        walk(else_branch, plugin)?;
                    }
                }
                Node::For(for_node) => walk(&mut for_node.body, plugin)?,
                Node::Match(match_node) => {
                    for case in &mut match_node.cases {
                        walk(&mut case.body, plugin)?;
                    }
                }
                Node::While(while_node) => walk(&mut while_node.body, plugin)?,
                Node::With(with_node) => walk(&mut with_node.body, plugin)?,
                Node::Try(try_node) => {
                    walk(&mut try_node.body, plugin)?;
                    for except in &mut try_node.except_clauses {
                        walk(&mut except.body, plugin)?;
                    }
                    if let Some(else_clause) = &mut try_node.else_clause {
                        walk(else_clause, plugin)?;
                    }
                    if let Some(finally_clause) = &mut try_node.finally_clause {
                        walk(finally_clause, plugin)?;
                    }
                }
                Node::Definition(def) => walk(&mut def.body, plugin)?,
                Node::Text(_)
                | Node::Expression(_)
                | Node::Comment(_)
                | Node::Statement(_)
                | Node::Import(_)
                | Node::Parameter(_)
                | Node::Decorator(_) => {}
            }
        }
        plugin.exit(node)?;
    }
    Ok(())
}

/// The standard plugins, in run order: transforms first, then inspectors.
pub fn standard_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(RenameReservedKeywords),
        Box::new(Async::default()),
        Box::new(Slots::default()),
        Box::new(MutableDefaults::default()),
        Box::new(SpreadKwargs::new()),
    ]
}

/// Run all standard plugins over the AST.
pub fn run(ast: &mut Ast) -> Result<(), CompileError> {
    for mut plugin in standard_plugins() {
        plugin.run(ast)?;
    }
    Ok(())
}
