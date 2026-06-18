use super::{Context, Flow, Helper, Plugin};
use crate::ast::{AttributeKind, Node};
use crate::error::CompileError;
use crate::html;

/// Detects which runtime helpers are needed based on AST structure.
/// The generator reads ctx.helpers_used to emit the correct imports.
pub struct HelperDetectionPlugin;

impl Plugin for HelperDetectionPlugin {
    fn enter(&mut self, node: &mut Node, ctx: &mut Context) -> Result<Flow, CompileError> {
        match node {
            Node::Expression(expr) => {
                ctx.helpers_used.insert(Helper::Escape);
                if expr.expr.contains("safe(") {
                    ctx.helpers_used.insert(Helper::Safe);
                }
            }
            Node::Element(el) => {
                for attr in &el.attributes {
                    // Reduce each attribute to (name, is_shorthand); spread/template insert directly.
                    let (name, is_shorthand) = match &attr.kind {
                        AttributeKind::Expression { name, .. } => (name.as_str(), false),
                        AttributeKind::Shorthand { name, .. } => (name.as_str(), true),
                        AttributeKind::Spread { .. } => {
                            ctx.helpers_used.insert(Helper::SpreadAttrs);
                            continue;
                        }
                        AttributeKind::Template { .. } => {
                            ctx.helpers_used.insert(Helper::Escape);
                            continue;
                        }
                        _ => continue,
                    };

                    let helper = match name {
                        "class" => Helper::RenderClass,
                        "style" => Helper::RenderStyle,
                        "data" => Helper::RenderData,
                        "aria" => Helper::RenderAria,
                        _ if is_shorthand => Helper::RenderAttr,
                        n if html::is_boolean_attribute(n) => Helper::RenderAttr,
                        _ => Helper::Escape,
                    };
                    ctx.helpers_used.insert(helper);
                }
            }
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
