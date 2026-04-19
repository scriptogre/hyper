use super::Visitor;
use super::metadata::Helper;
use crate::ast::{AttributeKind, Node};
use crate::html;

/// Detects which runtime helpers are needed based on AST structure.
/// The generator reads metadata.helpers_used to emit the correct imports.
pub struct HelperDetectionPlugin;

impl HelperDetectionPlugin {
    fn insert_helper_for_attr(name: &str, metadata: &mut super::TransformMetadata) {
        match name {
            "class" => metadata.helpers_used.insert(Helper::RenderClass),
            "style" => metadata.helpers_used.insert(Helper::RenderStyle),
            "data" => metadata.helpers_used.insert(Helper::RenderData),
            "aria" => metadata.helpers_used.insert(Helper::RenderAria),
            n if html::is_boolean_attribute(n) => metadata.helpers_used.insert(Helper::RenderAttr),
            _ => metadata.helpers_used.insert(Helper::Escape),
        };
    }
}

impl Visitor for HelperDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Expression(expr) => {
                metadata.helpers_used.insert(Helper::Escape);
                if expr.expr.contains("safe(") {
                    metadata.helpers_used.insert(Helper::Safe);
                }
            }
            Node::Element(el) => {
                for attr in &el.attributes {
                    match &attr.kind {
                        AttributeKind::Expression { name, .. } => {
                            Self::insert_helper_for_attr(name, metadata);
                        }
                        AttributeKind::Shorthand { name, .. } => match name.as_str() {
                            "class" | "style" | "data" | "aria" => {
                                Self::insert_helper_for_attr(name, metadata);
                            }
                            _ => {
                                metadata.helpers_used.insert(Helper::RenderAttr);
                            }
                        },
                        AttributeKind::Spread { .. } => {
                            metadata.helpers_used.insert(Helper::SpreadAttrs);
                        }
                        AttributeKind::Template { .. } => {
                            metadata.helpers_used.insert(Helper::Escape);
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
