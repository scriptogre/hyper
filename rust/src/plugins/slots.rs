use std::collections::BTreeMap;

use super::{Flow, Plugin, walk};
use crate::ast::{Function, Node, ParamKind, ParameterNode, TextRange};
use crate::error::{CompileError, ErrorKind, ParseError};

pub const DEFAULT_SLOT_PARAM: &str = "content";
const SLOT_TYPE_HINT: &str = "Iterable[str] | None";

/// Public Python argument for a slot.
pub fn slot_param_name(name: Option<&str>) -> String {
    name.unwrap_or(DEFAULT_SLOT_PARAM).to_string()
}

/// Adds keyword-only parameters for slots used by a component.
#[derive(Default)]
pub struct Slots {
    /// The empty name marks the default slot.
    names: BTreeMap<String, TextRange>,
}

impl Plugin for Slots {
    fn run(&mut self, function: &mut Function) -> Result<(), CompileError> {
        let declared: BTreeMap<String, TextRange> = function
            .params
            .iter()
            .filter_map(|node| match node {
                Node::Parameter(param) => {
                    Some((param.name.trim_start_matches('*').to_string(), param.range))
                }
                _ => None,
            })
            .collect();

        walk(&mut function.body, self)?;

        if let Some(range) = declared.get(DEFAULT_SLOT_PARAM) {
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                "`content` is reserved for the default slot.",
                *range,
            )
            .with_help("Rename this prop. Use `{...}` to render caller content.")
            .boxed()
            .into());
        }

        if let Some(range) = self.names.get(DEFAULT_SLOT_PARAM) {
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                "`content` names the default slot, not a named slot.",
                *range,
            )
            .with_help("Use `{...}` for the default slot, or rename this named slot.")
            .boxed()
            .into());
        }

        for (name, range) in self.names.iter().filter(|(name, _)| !name.is_empty()) {
            if let Some(prop_range) = declared.get(name) {
                return Err(ParseError::new(
                    ErrorKind::InvalidSyntax,
                    format!("`{name}` is both a prop and a named slot."),
                    *range,
                )
                .with_related(*prop_range)
                .with_related_label("prop declared here")
                .with_help("Rename the prop or the slot.")
                .boxed()
                .into());
            }
        }

        for name in self.names.keys() {
            function.params.push(Node::Parameter(ParameterNode {
                name: if name.is_empty() {
                    DEFAULT_SLOT_PARAM.to_string()
                } else {
                    slot_param_name(Some(name))
                },
                type_hint: Some(SLOT_TYPE_HINT.to_string()),
                default: Some("None".to_string()),
                kind: ParamKind::KeywordOnly,
                range: TextRange::synthetic(),
            }));
        }

        Ok(())
    }

    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            Node::Slot(slot) => {
                self.names
                    .insert(slot.name.clone().unwrap_or_default(), slot.range);
            }
            Node::Expression(expr) if expr.expr == "..." => {
                self.names.insert(String::new(), expr.range);
            }
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
