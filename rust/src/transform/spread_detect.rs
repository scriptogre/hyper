use std::collections::HashSet;

use super::Visitor;
use super::metadata::BLESSED_SPREAD_NAMES;
use crate::ast::{AttributeKind, Node};

/// Detects `{**name}` spread attributes with blessed names (kwargs, props, rest, attrs, attributes)
/// and records them for automatic injection into the function signature.
///
/// Names that are already declared as parameters (regular or **kwargs) are skipped.
/// Non-blessed names are ignored — they must already be in scope at runtime.
pub struct SpreadDetectionPlugin {
    declared_params: HashSet<String>,
}

impl SpreadDetectionPlugin {
    pub fn new() -> Self {
        Self {
            declared_params: HashSet::new(),
        }
    }
}

impl Default for SpreadDetectionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor for SpreadDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Parameter(param) => {
                let name = param.name.trim_start_matches('*');
                self.declared_params.insert(name.to_string());
            }
            Node::Element(el) => {
                self.check_spread_attrs(&el.attributes, metadata);
            }
            Node::Component(c) => {
                self.check_spread_attrs(&c.attributes, metadata);
            }
            _ => {}
        }
        true
    }
}

impl SpreadDetectionPlugin {
    fn check_spread_attrs(
        &self,
        attributes: &[crate::ast::Attribute],
        metadata: &mut super::TransformMetadata,
    ) {
        for attr in attributes {
            if let AttributeKind::Spread { expr, expr_span } = &attr.kind {
                let name = expr.trim();
                if self.declared_params.contains(name) {
                    continue;
                }
                if BLESSED_SPREAD_NAMES.contains(&name)
                    && !metadata.implicit_spreads.iter().any(|(n, _)| n == name)
                {
                    metadata
                        .implicit_spreads
                        .push((name.to_string(), *expr_span));
                }
            }
        }
    }
}
