use super::Visitor;
use crate::ast::{AttributeKind, Node};

/// Detects which helper functions are used in the template
/// This allows the generator to only import what's needed
pub struct HelperDetectionPlugin;

impl Visitor for HelperDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Expression(expr) => {
                // Track escape usage for escaped expressions
                if expr.escape {
                    metadata.helpers_used.insert("escape".to_string());
                }
                // Check if expression contains safe()
                if expr.expr.contains("safe(") {
                    metadata.helpers_used.insert("safe".to_string());
                }
            }
            Node::Element(el) => {
                // Check attributes for helper usage
                for attr in &el.attributes {
                    match &attr.kind {
                        AttributeKind::Dynamic { expr, .. } => {
                            if expr.contains("render_class(") {
                                metadata.helpers_used.insert("render_class".to_string());
                            }
                            if expr.contains("render_style(") {
                                metadata.helpers_used.insert("render_style".to_string());
                            }
                            if expr.contains("render_attr(") {
                                metadata.helpers_used.insert("render_attr".to_string());
                            }
                            if expr.contains("render_data(") {
                                metadata.helpers_used.insert("render_data".to_string());
                            }
                            if expr.contains("render_aria(") {
                                metadata.helpers_used.insert("render_aria".to_string());
                            }
                        }
                        AttributeKind::Spread { expr, .. } => {
                            if expr.contains("spread_attrs(") {
                                metadata.helpers_used.insert("spread_attrs".to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        true
    }
}
