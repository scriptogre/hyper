use super::{Context, Flow, Plugin, walk};
use crate::ast::{Ast, IfNode, Node, Span, StatementNode};
use crate::error::CompileError;

/// Rewrites mutable defaults on nullable params to the None-sentinel pattern.
///
/// A parameter declared `items: list | None = []` signals intent to use None as
/// the sentinel. This rewrites the default to `None` in place and prepends a
/// guard (`if items is None: items = []`) to the function body.
#[derive(Default)]
pub struct MutableDefaults {
    guards: Vec<(String, String)>,
}

impl Plugin for MutableDefaults {
    fn run(&mut self, ast: &mut Ast, ctx: &mut Context) -> Result<(), CompileError> {
        walk(&mut ast.function.params, ctx, self)?;
        walk(&mut ast.function.body, ctx, self)?;

        let guards = self.guards.iter().map(|(name, default)| {
            Node::If(IfNode {
                condition: format!("{name} is None"),
                condition_span: Span::synthetic(),
                then_branch: vec![Node::Statement(StatementNode {
                    stmt: format!("{name} = {default}"),
                    span: Span::synthetic(),
                })],
                elif_branches: Vec::new(),
                else_branch: None,
                span: Span::synthetic(),
            })
        });
        ast.function.body.splice(0..0, guards);

        Ok(())
    }

    fn enter(&mut self, node: &mut Node, _ctx: &mut Context) -> Result<Flow, CompileError> {
        if let Node::Parameter(param) = node
            && is_nullable_with_mutable_default(param.type_hint.as_deref(), param.default.as_deref())
            && let Some(default) = param.default.take()
        {
            self.guards.push((param.name.clone(), default));
            param.default = Some("None".to_string());
        }
        Ok(Flow::Continue)
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
