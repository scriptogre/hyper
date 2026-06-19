use std::collections::BTreeSet;

use super::{Context, Flow, Plugin, walk};
use crate::ast::{Ast, Node, ParamKind, ParameterNode, TextRange};
use crate::error::CompileError;

pub const DEFAULT_SLOT_PARAM: &str = "_default_slot";
const SLOT_TYPE_HINT: &str = "Iterable[str] | None";

/// Python identifier for a slot's parameter (`_default_slot`, `_header_slot`).
pub fn slot_param_name(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("_{n}_slot"),
        None => DEFAULT_SLOT_PARAM.to_string(),
    }
}

/// Lowers slots into signature parameters: the default slot becomes a positional
/// `_default_slot`, each named slot a keyword-only `_<name>_slot`. Slot usage in
/// the body is rendered separately by the generator.
#[derive(Default)]
pub struct Slots {
    /// Slot names used; the empty string marks the default slot.
    names: BTreeSet<String>,
}

impl Plugin for Slots {
    fn run(&mut self, ast: &mut Ast, ctx: &mut Context) -> Result<(), CompileError> {
        walk(&mut ast.function.params, ctx, self)?;
        walk(&mut ast.function.body, ctx, self)?;

        for name in &self.names {
            let (param_name, kind) = if name.is_empty() {
                (DEFAULT_SLOT_PARAM.to_string(), ParamKind::Positional)
            } else {
                (slot_param_name(Some(name)), ParamKind::KeywordOnly)
            };
            ast.function.params.push(Node::Parameter(ParameterNode {
                name: param_name,
                type_hint: Some(SLOT_TYPE_HINT.to_string()),
                default: Some("None".to_string()),
                kind,
                range: TextRange::synthetic(),
            }));
        }

        Ok(())
    }

    fn enter(&mut self, node: &mut Node, _ctx: &mut Context) -> Result<Flow, CompileError> {
        match node {
            Node::Slot(slot) => {
                self.names.insert(slot.name.clone().unwrap_or_default());
            }
            // {...} is the default children slot
            Node::Expression(expr) if expr.expr == "..." => {
                self.names.insert(String::new());
            }
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
