use super::Plugin;
use crate::ast::Node;

/// Detects parameters with nullable types and mutable defaults.
///
/// When a parameter is declared as `items: list | None = []`, the `| None`
/// signals intent to use the None sentinel pattern. This plugin records such
/// parameters so the generator can rewrite `= []` → `= None` with a guard.
pub struct MutableDefaultDetectionPlugin;

impl Plugin for MutableDefaultDetectionPlugin {
    fn enter(&mut self, node: &mut Node, metadata: &mut super::Analysis) -> bool {
        if let Node::Parameter(param) = node
            && is_nullable_with_mutable_default(
                param.type_hint.as_deref(),
                param.default.as_deref(),
            )
        {
            metadata.mutable_default_params.insert(param.name.clone());
        }
        true
    }
}

/// Check if a parameter has a nullable type hint and a mutable default value.
fn is_nullable_with_mutable_default(type_hint: Option<&str>, default: Option<&str>) -> bool {
    let Some(hint) = type_hint else {
        return false;
    };
    let Some(default) = default else {
        return false;
    };

    let is_nullable =
        hint.contains("| None") || hint.contains("None |") || hint.starts_with("Optional[");

    let is_mutable = default.starts_with('[')
        || default.starts_with('{')
        || default.starts_with("list(")
        || default.starts_with("dict(")
        || default.starts_with("set(");

    is_nullable && is_mutable
}
