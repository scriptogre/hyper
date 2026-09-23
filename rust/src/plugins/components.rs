use super::{Flow, Plugin, walk};
use crate::ast::{
    Ast, DecoratorNode, DefinitionKind, FileMode, FragmentNode, Function, FunctionDefinition, Node,
    ParamKind, ParameterNode, Position, ReturnAnnotation, TextRange,
};
use crate::error::{CompileError, ErrorKind, ParseError};
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
        let mut components = Self {
            library_root: ast.mode == FileMode::Library,
            ..Self::default()
        };
        components.run(&mut ast.function)?;
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
        if !self.library_root {
            self.runtime_imports.insert("component");
        }
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
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("tree-sitter Python language");
    let tree = parser
        .parse(&source, None)
        .expect("tree-sitter returned no tree");
    let root = tree.root_node();
    let function = root
        .named_child(0)
        .filter(|node| node.kind() == "function_definition" && !node.has_error())
        .ok_or_else(|| invalid_signature(definition.range))?;
    let name_node = function.child_by_field_name("name").expect("function name");
    let params_node = function
        .child_by_field_name("parameters")
        .expect("function parameters");
    let return_annotation =
        function
            .child_by_field_name("return_type")
            .map(|node| ReturnAnnotation {
                source: text(&source, node).to_string(),
                range: mapped_range(definition, node.start_byte(), node.end_byte()),
            });
    let name = text(&source, name_node).to_string();
    let name_range = mapped_range(definition, name_node.start_byte(), name_node.end_byte());
    let mut params = Vec::new();
    let mut keyword_only = false;
    let mut cursor = params_node.walk();

    for node in params_node.children(&mut cursor) {
        if !node.is_named() {
            match node.kind() {
                "*" => keyword_only = true,
                "/" => return Err(keyword_only_error(definition, node, &name)),
                _ => {}
            }
            continue;
        }

        match node.kind() {
            "keyword_separator" => {
                keyword_only = true;
                continue;
            }
            "positional_separator" => {
                return Err(keyword_only_error(definition, node, &name));
            }
            _ => {}
        }

        let (name_node, type_node, default_node) = match node.kind() {
            "identifier" => (node, None, None),
            "typed_parameter" => {
                let type_node = node.child_by_field_name("type").expect("parameter type");
                let name_node = first_named_child_except(node, type_node).expect("parameter name");
                (name_node, Some(type_node), None)
            }
            "default_parameter" => (
                node.child_by_field_name("name").expect("parameter name"),
                None,
                Some(
                    node.child_by_field_name("value")
                        .expect("parameter default"),
                ),
            ),
            "typed_default_parameter" => (
                node.child_by_field_name("name").expect("parameter name"),
                Some(node.child_by_field_name("type").expect("parameter type")),
                Some(
                    node.child_by_field_name("value")
                        .expect("parameter default"),
                ),
            ),
            "dictionary_splat_pattern" => (node, None, None),
            "list_splat_pattern" => {
                return Err(keyword_only_error(definition, node, &name));
            }
            _ => {
                return Err(invalid_signature(mapped_range(
                    definition,
                    node.start_byte(),
                    node.end_byte(),
                )));
            }
        };

        let kind = parameter_kind(name_node, definition, &name)?;
        if kind != ParamKind::VarKeyword && !keyword_only {
            return Err(keyword_only_error(definition, node, &name));
        }

        params.push(ParameterNode {
            name: text(&source, name_node).to_string(),
            type_hint: type_node.map(|value| text(&source, value).to_string()),
            default: default_node.map(|value| text(&source, value).to_string()),
            kind,
            range: mapped_range(definition, node.start_byte(), node.end_byte()),
        });
    }

    Ok(FunctionDefinition {
        name,
        name_range,
        return_annotation,
        function: Function {
            is_async,
            params: params.into_iter().map(Node::Parameter).collect(),
            imports: Vec::new(),
            decorators,
            header_comments: Vec::new(),
            body: std::mem::take(&mut definition.body),
        },
        range: definition.range,
    })
}

fn parameter_kind(
    node: tree_sitter::Node<'_>,
    definition: &crate::ast::DefinitionNode,
    component_name: &str,
) -> Result<ParamKind, CompileError> {
    match node.kind() {
        "identifier" => Ok(ParamKind::KeywordOnly),
        "dictionary_splat_pattern" => Ok(ParamKind::VarKeyword),
        "list_splat_pattern" => Err(keyword_only_error(definition, node, component_name)),
        _ => Err(invalid_signature(mapped_range(
            definition,
            node.start_byte(),
            node.end_byte(),
        ))),
    }
}

fn first_named_child_except<'tree>(
    node: tree_sitter::Node<'tree>,
    except: tree_sitter::Node<'tree>,
) -> Option<tree_sitter::Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.id() != except.id())
}

fn text<'a>(source: &'a str, node: tree_sitter::Node<'_>) -> &'a str {
    &source[node.byte_range()]
}

fn keyword_only_error(
    definition: &crate::ast::DefinitionNode,
    node: tree_sitter::Node<'_>,
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
        mapped_range(definition, node.start_byte(), node.end_byte()),
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

fn mapped_range(
    definition: &crate::ast::DefinitionNode,
    python_start: usize,
    python_end: usize,
) -> TextRange {
    TextRange {
        start: position_at(
            definition.signature_range.start,
            &definition.signature,
            python_start,
        ),
        end: position_at(
            definition.signature_range.start,
            &definition.signature,
            python_end,
        ),
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
