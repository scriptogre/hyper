use super::Visitor;
use crate::ast::{AttributeKind, Node};
use crate::html;

/// Detects which runtime helpers are needed based on AST structure.
/// The generator reads metadata.helpers_used to emit the correct imports.
pub struct HelperDetectionPlugin;

impl Visitor for HelperDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Expression(expr) => {
                metadata.helpers_used.insert("escape".to_string());
                if expr.expr.contains("safe(") {
                    metadata.helpers_used.insert("safe".to_string());
                }
            }
            Node::Element(el) => {
                for attr in &el.attributes {
                    match &attr.kind {
                        AttributeKind::Dynamic { name, .. } => match name.as_str() {
                            "class" => {
                                metadata.helpers_used.insert("render_class".to_string());
                            }
                            "style" => {
                                metadata.helpers_used.insert("render_style".to_string());
                            }
                            "data" => {
                                metadata.helpers_used.insert("render_data".to_string());
                            }
                            "aria" => {
                                metadata.helpers_used.insert("render_aria".to_string());
                            }
                            n if html::is_boolean_attribute(n) => {
                                metadata.helpers_used.insert("render_attr".to_string());
                            }
                            _ => {
                                metadata.helpers_used.insert("escape".to_string());
                            }
                        },
                        AttributeKind::Shorthand { name, .. } => match name.as_str() {
                            "class" => {
                                metadata.helpers_used.insert("render_class".to_string());
                            }
                            "style" => {
                                metadata.helpers_used.insert("render_style".to_string());
                            }
                            "data" => {
                                metadata.helpers_used.insert("render_data".to_string());
                            }
                            "aria" => {
                                metadata.helpers_used.insert("render_aria".to_string());
                            }
                            _ => {
                                metadata.helpers_used.insert("render_attr".to_string());
                            }
                        },
                        AttributeKind::Spread { .. } => {
                            metadata.helpers_used.insert("spread_attrs".to_string());
                        }
                        AttributeKind::Template { .. } => {
                            metadata.helpers_used.insert("escape".to_string());
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
