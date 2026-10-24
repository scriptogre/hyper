use super::{Flow, Plugin};
use crate::ast::{AttributeKind, Function, Node, ParamKind, ParameterNode, TextRange};
use crate::error::CompileError;

/// Spread names injected into the signature when used as `{**name}` undeclared.
const BLESSED_SPREAD_NAMES: &[&str] = &["kwargs", "props", "rest", "attrs", "attributes"];

/// Auto-injects a `**kwargs` parameter for blessed spread names (kwargs, props,
/// rest, attrs, attributes) used as `{**name}` without an explicit declaration.
///
/// Inspect (`enter`): records blessed spread names used.
///
/// Transform (`finish`): drops declared names, rejects more than one distinct
/// blessed name, then appends a `**name` parameter so the signature carries it.
#[derive(Default)]
pub struct SpreadKwargs {
    blessed_spreads: Vec<String>,
}

impl Plugin for SpreadKwargs {
    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        let attributes = match node {
            Node::Element(el) => &el.attributes,
            Node::Component(c) => &c.attributes,
            _ => return Ok(Flow::Continue),
        };
        for attr in attributes {
            if let AttributeKind::Spread { expr, .. } = &attr.kind {
                let name = expr.trim();
                if BLESSED_SPREAD_NAMES.contains(&name)
                    && !self.blessed_spreads.iter().any(|n| n == name)
                {
                    self.blessed_spreads.push(name.to_string());
                }
            }
        }
        Ok(Flow::Continue)
    }

    // Runs after Slots::finish, so slot params count as declared.
    fn finish(&mut self, function: &mut Function) -> Result<(), CompileError> {
        let params: Vec<&str> = function
            .params
            .iter()
            .filter_map(|node| match node {
                Node::Parameter(param) => Some(param.name.as_str()),
                _ => None,
            })
            .collect();
        self.blessed_spreads
            .retain(|name| !params.iter().any(|p| p.trim_start_matches('*') == name));
        let has_explicit_kwargs = params.iter().any(|p| p.starts_with("**"));

        // Guard: only one distinct blessed spread name is allowed per template.
        if self.blessed_spreads.len() > 1 {
            let names_list = self
                .blessed_spreads
                .iter()
                .map(|n| format!("{{**{n}}}"))
                .collect::<Vec<_>>()
                .join(" and ");
            return Err(CompileError::Generate(format!(
                "Cannot use {names_list} in the same template \u{2014} only one spread parameter is allowed per component"
            )));
        }

        // Transform: inject `**name` into the signature, unless one is declared.
        if !has_explicit_kwargs && let Some(name) = self.blessed_spreads.first() {
            function.params.push(Node::Parameter(ParameterNode {
                name: format!("**{name}"),
                type_hint: None,
                default: None,
                kind: ParamKind::VarKeyword,
                range: TextRange::synthetic(),
            }));
        }

        Ok(())
    }
}
