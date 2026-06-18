use std::collections::HashSet;

use super::context::BLESSED_SPREAD_NAMES;
use super::{Context, Flow, Plugin, walk};
use crate::ast::{Ast, Attribute, AttributeKind, Node};
use crate::error::CompileError;

/// Handles `{**name}` spread attributes with blessed names (kwargs, props, rest,
/// attrs, attributes).
///
/// Inspect (`enter`): records blessed spread names for auto-injection into the
/// function signature, skipping names already declared as parameters.
///
/// Guard (`run`): after the walk, rejects templates that use more than one
/// distinct blessed spread name.
pub struct SpreadKwargsPlugin {
    declared_params: HashSet<String>,
}

impl SpreadKwargsPlugin {
    pub fn new() -> Self {
        Self {
            declared_params: HashSet::new(),
        }
    }

    fn collect_blessed_spreads(&self, attributes: &[Attribute], ctx: &mut Context) {
        for attr in attributes {
            if let AttributeKind::Spread { expr, expr_span } = &attr.kind {
                let name = expr.trim();
                if self.declared_params.contains(name) {
                    continue;
                }
                if BLESSED_SPREAD_NAMES.contains(&name)
                    && !ctx.implicit_spreads.iter().any(|(n, _)| n == name)
                {
                    ctx.implicit_spreads.push((name.to_string(), *expr_span));
                }
            }
        }
    }
}

impl Default for SpreadKwargsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SpreadKwargsPlugin {
    fn run(&mut self, ast: &mut Ast, ctx: &mut Context) -> Result<(), CompileError> {
        walk(&mut ast.nodes, ctx, self)?;

        // Guard: only one distinct blessed spread name is allowed per template.
        let mut unique_names: Vec<&str> = Vec::new();
        for (name, _) in &ctx.implicit_spreads {
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

    fn enter(&mut self, node: &mut Node, ctx: &mut Context) -> Result<Flow, CompileError> {
        match node {
            Node::Parameter(param) => {
                let name = param.name.trim_start_matches('*');
                self.declared_params.insert(name.to_string());
            }
            Node::Element(el) => self.collect_blessed_spreads(&el.attributes, ctx),
            Node::Component(c) => self.collect_blessed_spreads(&c.attributes, ctx),
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
