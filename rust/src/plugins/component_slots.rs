use std::collections::BTreeMap;

use super::{Flow, Plugin};
use crate::ast::{Attribute, AttributeKind, ComponentNode, Node, TextRange};
use crate::error::{CompileError, ErrorKind, ParseError};
use crate::plugins::DEFAULT_SLOT_PARAM;

/// Binds caller-side named slot syntax to component invocations.
#[derive(Default)]
pub struct ComponentSlots;

impl Plugin for ComponentSlots {
    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            Node::Component(component) => {
                bind_slots(component)?;
                validate_bound_assignments(&component.attributes)?;
            }
            Node::Element(element) => validate_bound_assignments(&element.attributes)?,
            _ => {}
        }
        Ok(Flow::Continue)
    }
}

fn bind_slots(component: &mut ComponentNode) -> Result<(), CompileError> {
    let mut children = Vec::with_capacity(component.children.len());
    let mut ranges = BTreeMap::new();

    for mut child in std::mem::take(&mut component.children) {
        let explicit_fill = match &mut child {
            Node::Slot(slot) if slot.name.is_some() && slot.close_range.is_some() => {
                slot.is_fill = true;
                Some((slot.name.clone().expect("named slot"), slot.range))
            }
            _ => None,
        };
        let assignment = if explicit_fill.is_none() {
            bind_assignment(&mut child)?
        } else {
            None
        };

        if let Some((name, range)) = explicit_fill.or(assignment) {
            validate_slot_name(&name, range)?;
            if let Some(first_range) = ranges.get(&name) {
                return Err(ParseError::new(
                    ErrorKind::InvalidSyntax,
                    format!("The `{name}` slot is filled more than once."),
                    range,
                )
                .with_related(*first_range)
                .with_related_label("first fill")
                .with_help("Wrap all content for this slot in one named slot block.")
                .boxed()
                .into());
            }
            ranges.insert(name.clone(), range);
            component.slots.insert(name, vec![child]);
        } else {
            children.push(child);
        }
    }

    component.children = children;
    Ok(())
}

fn bind_assignment(node: &mut Node) -> Result<Option<(String, TextRange)>, CompileError> {
    let attributes = match node {
        Node::Element(element) => &mut element.attributes,
        Node::Component(component) => &mut component.attributes,
        _ => return Ok(None),
    };
    let mut assignment = None;

    for attribute in attributes {
        let AttributeKind::SlotAssignment { name, bound, .. } = &mut attribute.kind else {
            continue;
        };
        if let Some((_, first_range)) = &assignment {
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                "An element can fill only one named slot.",
                attribute.range,
            )
            .with_related(*first_range)
            .with_related_label("first slot")
            .with_help("Remove one slot marker, or use explicit named slot blocks.")
            .boxed()
            .into());
        }
        *bound = true;
        assignment = Some((name.clone(), attribute.range));
    }

    Ok(assignment)
}

fn validate_slot_name(name: &str, range: TextRange) -> Result<(), CompileError> {
    if name.is_empty() {
        return Err(ParseError::new(
            ErrorKind::InvalidSyntax,
            "A slot fill must have a name.",
            range,
        )
        .with_help("Remove the marker for default content, or use `{...name}`.")
        .boxed()
        .into());
    }
    if name == DEFAULT_SLOT_PARAM {
        return Err(ParseError::new(
            ErrorKind::InvalidSyntax,
            "`content` names the default slot, not a named slot.",
            range,
        )
        .with_help("Remove the marker for default content, or rename this named slot.")
        .boxed()
        .into());
    }
    Ok(())
}

fn validate_bound_assignments(attributes: &[Attribute]) -> Result<(), CompileError> {
    for attribute in attributes {
        if let AttributeKind::SlotAssignment {
            name, bound: false, ..
        } = &attribute.kind
        {
            return Err(ParseError::new(
                ErrorKind::InvalidSyntax,
                format!("The `{name}` slot marker is not a direct component child."),
                attribute.range,
            )
            .with_help("Move this element directly inside the component call.")
            .boxed()
            .into());
        }
    }
    Ok(())
}
