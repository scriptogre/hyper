use super::Visitor;
use crate::ast::{AttributeKind, Node};

/// Detects implicit spread usage ({**name} without a declared **name parameter)
pub struct SpreadDetectionPlugin;

impl Visitor for SpreadDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        if let Node::Element(el) = node {
            for attr in &el.attributes {
                if let AttributeKind::Spread { expr, .. } = &attr.kind {
                    metadata.spread_names.insert(expr.trim().to_string());
                }
            }
        }
        true
    }
}
