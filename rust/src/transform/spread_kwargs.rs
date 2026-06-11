use std::collections::HashSet;

use super::Plugin;
use super::metadata::BLESSED_SPREAD_NAMES;
use crate::ast::{AttributeKind, Node};
use crate::error::CompileError;

/// Handles `{**name}` spread attributes with blessed names (kwargs, props, rest,
/// attrs, attributes).
///
/// Scan: records blessed spread names for auto-injection into the function
/// signature. Skips names already declared as parameters.
///
/// Guard: rejects templates that use more than one distinct blessed spread name.
pub struct SpreadKwargsPlugin {
    declared_params: HashSet<String>,
}

impl SpreadKwargsPlugin {
    pub fn new() -> Self {
        Self {
            declared_params: HashSet::new(),
        }
    }
}

impl Default for SpreadKwargsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SpreadKwargsPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Parameter(param) => {
                let name = param.name.trim_start_matches('*');
                self.declared_params.insert(name.to_string());
            }
            Node::Element(el) => {
                self.collect_blessed_spreads(&el.attributes, metadata);
            }
            Node::Component(c) => {
                self.collect_blessed_spreads(&c.attributes, metadata);
            }
            _ => {}
        }
        true
    }

    fn finalize(&mut self, metadata: &super::TransformMetadata) -> Result<(), CompileError> {
        let mut unique_names: Vec<&str> = Vec::new();
        for (name, _) in &metadata.implicit_spreads {
            if !unique_names.contains(&name.as_str()) {
                unique_names.push(name);
            }
        }

        if unique_names.len() > 1 {
            let names_list = unique_names
                .iter()
                .map(|n| format!("{{**{n}}}"))
                .collect::<Vec<_>>()
                .join(" and ");
            return Err(CompileError::Generate(format!(
                "Cannot use {names_list} in the same template \u{2014} only one spread parameter is allowed per component"
            )));
        }

        Ok(())
    }
}

impl SpreadKwargsPlugin {
    fn collect_blessed_spreads(
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
