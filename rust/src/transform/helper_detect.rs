use super::Visitor;
use crate::ast::{AttributeKind, Node};

/// Detects which helper functions are used in the template
/// This allows the generator to only import what's needed
pub struct HelperDetectionPlugin;

impl Visitor for HelperDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Expression(expr)
                // Note: escape() import is detected by the generator via code.contains("{escape(")
                // rather than being tracked as a helper here
                //
                // Check if expression explicitly uses safe()
                if expr.expr.contains("safe(") => {
                    metadata.helpers_used.insert("safe".to_string());
            }
            Node::Element(el) => {
                // Check attributes for helper usage
                for attr in &el.attributes {
                    if let AttributeKind::Dynamic { expr, .. } = &attr.kind {
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
                }
            }
            _ => {}
        }
        true
    }
}
