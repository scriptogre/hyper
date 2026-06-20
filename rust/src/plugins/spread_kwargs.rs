use std::collections::HashSet;

use super::context::BLESSED_SPREAD_NAMES;
use super::{Flow, Plugin, walk};
use crate::ast::{Ast, Attribute, AttributeKind, Node, ParamKind, ParameterNode, TextRange};
use crate::error::CompileError;

/// Auto-injects a `**kwargs` parameter for blessed spread names (kwargs, props,
/// rest, attrs, attributes) used as `{**name}` without an explicit declaration.
///
/// Inspect (`enter`): records declared params and blessed spread names used.
///
/// Transform (`run`): after the walk, rejects more than one distinct blessed name,
/// then appends a `**name` parameter so the signature carries it.
pub struct SpreadKwargs {
    declared_params: HashSet<String>,
    has_explicit_kwargs: bool,
    blessed_spreads: Vec<(String, TextRange)>,
}

impl SpreadKwargs {
    pub fn new() -> Self {
        Self {
            declared_params: HashSet::new(),
            has_explicit_kwargs: false,
            blessed_spreads: Vec::new(),
        }
    }

    fn collect_blessed_spreads(&mut self, attributes: &[Attribute]) {
        for attr in attributes {
            if let AttributeKind::Spread { expr, expr_range } = &attr.kind {
                let name = expr.trim();
                if self.declared_params.contains(name) {
                    continue;
                }
                if BLESSED_SPREAD_NAMES.contains(&name)
                    && !self.blessed_spreads.iter().any(|(n, _)| n == name)
                {
                    self.blessed_spreads.push((name.to_string(), *expr_range));
                }
            }
        }
    }
}

impl Default for SpreadKwargs {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SpreadKwargs {
    fn run(&mut self, ast: &mut Ast) -> Result<(), CompileError> {
        walk(&mut ast.function.params, self)?;
        walk(&mut ast.function.body, self)?;

        // Guard: only one distinct blessed spread name is allowed per template.
        if self.blessed_spreads.len() > 1 {
            let names_list = self
                .blessed_spreads
                .iter()
                .map(|(n, _)| format!("{{**{n}}}"))
                .collect::<Vec<_>>()
                .join(" and ");
            return Err(CompileError::Generate(format!(
                "Cannot use {names_list} in the same template \u{2014} only one spread parameter is allowed per component"
            )));
        }

        // Transform: inject `**name` into the signature, unless one is declared.
        if !self.has_explicit_kwargs
            && let Some((name, _)) = self.blessed_spreads.first()
        {
            ast.function.params.push(Node::Parameter(ParameterNode {
                name: format!("**{name}"),
                type_hint: None,
                default: None,
                kind: ParamKind::VarKeyword,
                range: TextRange::synthetic(),
            }));
        }

        Ok(())
    }

    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            Node::Parameter(param) => {
                if param.name.starts_with("**") {
                    self.has_explicit_kwargs = true;
                }
                let name = param.name.trim_start_matches('*');
                self.declared_params.insert(name.to_string());
            }
            Node::Element(el) => self.collect_blessed_spreads(&el.attributes),
            Node::Component(c) => self.collect_blessed_spreads(&c.attributes),
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
