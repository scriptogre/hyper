use super::Plugin;
use crate::ast::Node;

/// Detects slots used in the template
pub struct SlotDetectionPlugin;

impl Plugin for SlotDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::Analysis) -> bool {
        match node {
            Node::Slot(slot) => {
                // Default slot uses empty string, named slots use their name
                let slot_name = slot.name.clone().unwrap_or_default();
                metadata.slots_used.insert(slot_name);
            }
            Node::Expression(expr)
                // Check for {...} which is the default children slot
                if expr.expr == "..." => {
                    metadata.slots_used.insert(String::new());
            }
            _ => {}
        }
        true
    }
}
