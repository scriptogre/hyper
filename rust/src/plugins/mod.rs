mod async_detect;
mod context;
mod helper_detect;
mod mutable_default_detect;
mod reserved_keyword;
mod slot_detect;
mod spread_kwargs;

pub use async_detect::AsyncDetectionPlugin;
pub use context::{BLESSED_SPREAD_NAMES, Context, Helper};
pub use helper_detect::HelperDetectionPlugin;
pub use mutable_default_detect::MutableDefaultDetectionPlugin;
pub use reserved_keyword::{ReservedKeywordPlugin, rename_reserved_keywords};
pub use slot_detect::SlotDetectionPlugin;
pub use spread_kwargs::SpreadKwargsPlugin;

use crate::ast::{Ast, Node};
use crate::error::CompileError;

/// Whether [`walk`] descends into a node's children after `enter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    SkipChildren,
}

/// A compiler plugin. Reads and rewrites the AST, and fills the shared [`Context`].
///
/// Override `enter`/`exit` for per-node work (the common case). Override `run`
/// to own the traversal: walk twice, reorder nodes via `&mut Ast`, or guard
/// after the walk. Local state lives on the plugin struct; shared state in [`Context`].
pub trait Plugin {
    /// Run the plugin over the whole tree. Default walks top-down, calling
    /// `enter` then `exit` on each node.
    fn run(&mut self, ast: &mut Ast, ctx: &mut Context) -> Result<(), CompileError> {
        walk(&mut ast.nodes, ctx, self)
    }

    /// Called before a node's children. Return [`Flow::SkipChildren`] to prune.
    fn enter(&mut self, _node: &mut Node, _ctx: &mut Context) -> Result<Flow, CompileError> {
        Ok(Flow::Continue)
    }

    /// Called after a node's children.
    fn exit(&mut self, _node: &mut Node, _ctx: &mut Context) -> Result<(), CompileError> {
        Ok(())
    }
}

/// Recurse the tree, calling `plugin.enter` (then `exit`) on each node. The one
/// place that knows the AST shape; plugins reuse it instead of reimplementing it.
pub fn walk<P: Plugin + ?Sized>(
    nodes: &mut [Node],
    ctx: &mut Context,
    plugin: &mut P,
) -> Result<(), CompileError> {
    for node in nodes {
        if plugin.enter(node, ctx)? == Flow::Continue {
            match node {
                Node::Element(el) => walk(&mut el.children, ctx, plugin)?,
                Node::Component(c) => {
                    walk(&mut c.children, ctx, plugin)?;
                    for slot in c.slots.values_mut() {
                        walk(slot, ctx, plugin)?;
                    }
                }
                Node::Fragment(f) => walk(&mut f.children, ctx, plugin)?,
                Node::Slot(s) => walk(&mut s.fallback, ctx, plugin)?,
                Node::If(if_node) => {
                    walk(&mut if_node.then_branch, ctx, plugin)?;
                    for (_, _, branch) in &mut if_node.elif_branches {
                        walk(branch, ctx, plugin)?;
                    }
                    if let Some(else_branch) = &mut if_node.else_branch {
                        walk(else_branch, ctx, plugin)?;
                    }
                }
                Node::For(for_node) => walk(&mut for_node.body, ctx, plugin)?,
                Node::Match(match_node) => {
                    for case in &mut match_node.cases {
                        walk(&mut case.body, ctx, plugin)?;
                    }
                }
                Node::While(while_node) => walk(&mut while_node.body, ctx, plugin)?,
                Node::With(with_node) => walk(&mut with_node.body, ctx, plugin)?,
                Node::Try(try_node) => {
                    walk(&mut try_node.body, ctx, plugin)?;
                    for except in &mut try_node.except_clauses {
                        walk(&mut except.body, ctx, plugin)?;
                    }
                    if let Some(else_clause) = &mut try_node.else_clause {
                        walk(else_clause, ctx, plugin)?;
                    }
                    if let Some(finally_clause) = &mut try_node.finally_clause {
                        walk(finally_clause, ctx, plugin)?;
                    }
                }
                Node::Definition(def) => walk(&mut def.body, ctx, plugin)?,
                Node::Text(_)
                | Node::Expression(_)
                | Node::Comment(_)
                | Node::Statement(_)
                | Node::Import(_)
                | Node::Parameter(_)
                | Node::Decorator(_) => {}
            }
        }
        plugin.exit(node, ctx)?;
    }
    Ok(())
}

/// The standard plugins, in run order: transforms first, then inspectors.
pub fn standard_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(ReservedKeywordPlugin),
        Box::new(HelperDetectionPlugin),
        Box::new(AsyncDetectionPlugin),
        Box::new(SlotDetectionPlugin),
        Box::new(MutableDefaultDetectionPlugin),
        Box::new(SpreadKwargsPlugin::new()),
    ]
}

/// Run all standard plugins over the AST, returning the shared context.
pub fn run(ast: &mut Ast) -> Result<Context, CompileError> {
    let mut ctx = Context::new();
    for mut plugin in standard_plugins() {
        plugin.run(ast, &mut ctx)?;
    }
    Ok(ctx)
}
