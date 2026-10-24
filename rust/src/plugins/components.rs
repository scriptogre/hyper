use super::{Flow, Plugin, walk};
use crate::ast::{
    Ast, DecoratorNode, DefinitionKind, FragmentNode, Function, FunctionDefinition, Node,
    ParamKind, ParameterNode, Position, ReturnAnnotation, TextRange,
};
use crate::error::{CompileError, ErrorKind, ParseError};
use ruff_python_ast::Stmt;
use ruff_text_size::{Ranged, TextRange as PyRange};
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Components {
    definitions: Vec<FunctionDefinition>,
    scopes: Vec<Scope>,
    pending_subcomponent: Option<DecoratorNode>,
    library_root: bool,
    runtime_imports: BTreeSet<&'static str>,
}

#[derive(Default)]
struct Scope {
    has_output: bool,
    subcomponent: Option<DecoratorNode>,
    children: Vec<String>,
}

impl Components {
    pub fn lower(ast: &mut Ast) -> Result<(), CompileError> {
        // Module-level templates become exported definitions.
        let mut components = Self {
            library_root: true,
            ..Self::default()
        };
        components.scopes.push(Scope::default());
        walk(&mut ast.module.body, &mut components)?;
        components.scopes.pop();

        if let Some(component) = &mut ast.file_component {
            components.library_root = false;
            components.run(&mut component.function)?;
        }
        ast.runtime_imports = ["component"]
            .into_iter()
            .filter(|name| components.runtime_imports.contains(name))
            .collect();
        ast.definitions = components.definitions;
        Ok(())
    }
}

impl Plugin for Components {
    fn run(&mut self, function: &mut Function) -> Result<(), CompileError> {
        self.runtime_imports.insert("component");
        self.scopes.push(Scope::default());
        walk(&mut function.body, self)?;
        let root = self.scopes.pop().expect("root component scope");
        function
            .decorators
            .push(component_decorator(&root.children));
        Ok(())
    }

    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            Node::Decorator(decorator) if decorator.decorator.trim() == "@subcomponent" => {
                let range = decorator.range;
                self.pending_subcomponent = Some(decorator.clone());
                *node = Node::Fragment(FragmentNode {
                    children: Vec::new(),
                    range,
                });
                Ok(Flow::Continue)
            }
            Node::Definition(_) => {
                self.scopes.push(Scope {
                    subcomponent: self.pending_subcomponent.take(),
                    ..Scope::default()
                });
                Ok(Flow::Continue)
            }
            Node::Text(text) if !text.content.trim().is_empty() => {
                self.scopes.last_mut().expect("component scope").has_output = true;
                Ok(Flow::Continue)
            }
            Node::Expression(_) | Node::Element(_) | Node::Component(_) | Node::Slot(_) => {
                self.scopes.last_mut().expect("component scope").has_output = true;
                Ok(Flow::Continue)
            }
            Node::Comment(_) | Node::Text(_) | Node::Fragment(_) => Ok(Flow::Continue),
            _ => {
                self.pending_subcomponent = None;
                Ok(Flow::Continue)
            }
        }
    }

    fn exit(&mut self, node: &mut Node) -> Result<(), CompileError> {
        let Node::Definition(definition) = node else {
            return Ok(());
        };
        let scope = self.scopes.pop().expect("definition scope");
        let is_template = has_component_annotation(definition) && scope.has_output;
        if !is_template {
            self.scopes
                .last_mut()
                .expect("parent component scope")
                .children
                .extend(scope.children);
            return Ok(());
        }

        let is_subcomponent = scope.subcomponent.is_some();
        self.runtime_imports.insert("component");
        let mut decorators = vec![component_decorator(&scope.children)];
        if let Some(subcomponent) = scope.subcomponent {
            decorators.insert(0, subcomponent);
        }
        let lowered = lower_definition(definition, decorators)?;
        let name = lowered.name.clone();
        let range = lowered.range;

        let is_module = self.library_root && self.scopes.len() == 1 || is_subcomponent;
        if is_module {
            self.definitions.push(lowered);
            *node = Node::Fragment(FragmentNode {
                children: Vec::new(),
                range,
            });
        } else {
            *node = Node::Function(lowered);
        }
        if is_subcomponent {
            self.scopes
                .last_mut()
                .expect("parent component scope")
                .children
                .push(name);
        }
        Ok(())
    }
}

fn has_component_annotation(definition: &crate::ast::DefinitionNode) -> bool {
    definition.kind == DefinitionKind::Function
        && definition
            .signature
            .trim_end_matches(':')
            .rsplit_once("->")
            .is_some_and(|(_, annotation)| annotation.trim() == "Component")
}

fn component_decorator(children: &[String]) -> DecoratorNode {
    let decorator = if children.is_empty() {
        "@component".to_string()
    } else {
        format!("@component(subcomponents=[{}])", children.join(", "))
    };
    DecoratorNode {
        decorator,
        range: TextRange::synthetic(),
    }
}

fn lower_definition(
    definition: &mut crate::ast::DefinitionNode,
    decorators: Vec<DecoratorNode>,
) -> Result<FunctionDefinition, CompileError> {
    let signature = definition.signature.trim_start();
    debug_assert!(has_component_annotation(definition));
    let is_async = signature.starts_with("async def ");
    let source = format!("{signature}\n    pass");
    let Some(Stmt::FunctionDef(function)) = ruff_python_parser::parse_module(&source)
        .ok()
        .and_then(|parsed| parsed.into_syntax().body.into_iter().next())
    else {
        return Err(invalid_signature(definition.range));
    };
    let name = function.name.id.to_string();
    let parameters = &function.parameters;

    let positional = parameters
        .posonlyargs
        .iter()
        .chain(&parameters.args)
        .map(Ranged::range)
        .chain(parameters.vararg.as_deref().map(Ranged::range))
        .next();
    if let Some(range) = positional {
        return Err(keyword_only_error(definition, range, &name));
    }

    let text = |range: PyRange| source[range].to_string();
    let keyword_only = parameters.kwonlyargs.iter().map(|param| ParameterNode {
        name: param.name().id.to_string(),
        type_hint: param
            .annotation()
            .map(|annotation| text(annotation.range())),
        default: param.default().map(|default| text(default.range())),
        kind: ParamKind::KeywordOnly,
        range: mapped_range(definition, param.range()),
    });
    let var_keyword = parameters.kwarg.as_deref().map(|kwarg| ParameterNode {
        name: format!("**{}", kwarg.name.id),
        type_hint: kwarg
            .annotation()
            .map(|annotation| text(annotation.range())),
        default: None,
        kind: ParamKind::VarKeyword,
        range: mapped_range(definition, kwarg.range()),
    });
    let params = keyword_only
        .chain(var_keyword)
        .map(Node::Parameter)
        .collect();
    let return_annotation = function.returns.as_deref().map(|returns| ReturnAnnotation {
        source: text(returns.range()),
        range: mapped_range(definition, returns.range()),
    });

    Ok(FunctionDefinition {
        name,
        name_range: mapped_range(definition, function.name.range()),
        return_annotation,
        function: Function {
            is_async,
            params,
            imports: Vec::new(),
            decorators,
            header_comments: Vec::new(),
            body: std::mem::take(&mut definition.body),
        },
        range: definition.range,
    })
}

fn keyword_only_error(
    definition: &crate::ast::DefinitionNode,
    range: PyRange,
    name: &str,
) -> CompileError {
    let parameters = definition
        .signature
        .split_once('(')
        .and_then(|(_, rest)| rest.rsplit_once(')'))
        .map(|(params, _)| params.trim())
        .unwrap_or_default();
    ParseError::new(
        ErrorKind::InvalidSyntax,
        "Component props must be keyword-only.",
        mapped_range(definition, range),
    )
    .with_help(format!(
        "Add `*,` before the first prop:\n\n  def {name}(*, {parameters}) -> Component:"
    ))
    .boxed()
    .into()
}

fn invalid_signature(range: TextRange) -> CompileError {
    ParseError::new(
        ErrorKind::InvalidSyntax,
        "This component signature is invalid.",
        range,
    )
    .with_help("Use `def Name() -> Component:` or `def Name(*, prop: Type) -> Component:`.")
    .boxed()
    .into()
}

fn mapped_range(definition: &crate::ast::DefinitionNode, range: PyRange) -> TextRange {
    let base = definition.signature_range.start;
    TextRange {
        start: position_at(base, &definition.signature, range.start().to_usize()),
        end: position_at(base, &definition.signature, range.end().to_usize()),
    }
}

fn position_at(base: Position, source: &str, offset: usize) -> Position {
    let mut line = base.line;
    let mut col = base.col;
    for ch in source[..offset].chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Position {
        byte: base.byte + offset,
        line,
        col,
    }
}
