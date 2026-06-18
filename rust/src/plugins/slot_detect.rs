use super::{Context, Flow, Plugin};
use crate::ast::Node;
use crate::error::CompileError;

/// Detects slots used in the template
pub struct SlotDetectionPlugin;

impl Plugin for SlotDetectionPlugin {
    fn enter(&mut self, node: &mut Node, ctx: &mut Context) -> Result<Flow, CompileError> {
        match node {
            Node::Slot(slot) => {
                // Default slot uses empty string, named slots use their name
                let slot_name = slot.name.clone().unwrap_or_default();
                ctx.slots_used.insert(slot_name);
            }
            // {...} is the default children slot
            Node::Expression(expr) if expr.expr == "..." => {
                ctx.slots_used.insert(String::new());
            }
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
