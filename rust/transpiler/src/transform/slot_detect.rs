use super::Visitor;
use crate::ast::Node;

/// Detects slots used in the template
pub struct SlotDetectionPlugin;

impl Visitor for SlotDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Slot(slot) => {
                // Default slot uses empty string, named slots use their name
                let slot_name = slot.name.clone().unwrap_or_else(|| String::new());
                metadata.slots_used.insert(slot_name);
            }
            Node::Expression(expr) => {
                // Check for {...} which is the default children slot
                if expr.expr == "..." {
                    metadata.slots_used.insert(String::new());
                }
            }
            _ => {}
        }
        true
    }
}
