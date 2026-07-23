use super::{Flow, Plugin, walk};
use crate::ast::{
    DecoratorNode, DefinitionKind, FragmentNode, Function, FunctionDefinition, Node, ParamKind,
    ParameterNode, Position, TextRange,
};
use crate::error::{CompileError, ErrorKind, ParseError};

#[derive(Default)]
pub struct Components {
    children: Vec<Vec<String>>,
    definitions: Vec<FunctionDefinition>,
}

impl Components {
    pub fn into_definitions(self) -> Vec<FunctionDefinition> {
        self.definitions
    }
}

impl Plugin for Components {
    fn run(&mut self, function: &mut Function) -> Result<(), CompileError> {
        self.children.push(Vec::new());
        walk(&mut function.body, self)?;
        let children = self.children.pop().expect("root component scope");
        function.decorators.push(component_decorator(&children));
        Ok(())
    }

    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            Node::Definition(definition) if definition.kind == DefinitionKind::Component => {
                self.children.push(Vec::new());
                Ok(Flow::Continue)
            }
            Node::Definition(_) => Ok(Flow::SkipChildren),
            _ => Ok(Flow::Continue),
        }
    }

    fn exit(&mut self, node: &mut Node) -> Result<(), CompileError> {
        let Node::Definition(definition) = node else {
            return Ok(());
        };
        if definition.kind != DefinitionKind::Component {
            return Ok(());
        }

        let children = self.children.pop().expect("component scope");
        let (name, name_range, params, is_async) = parse_signature(definition)?;
        let range = definition.range;
        self.definitions.push(FunctionDefinition {
            name: name.clone(),
            name_range,
            function: Function {
                is_async,
                params: params.into_iter().map(Node::Parameter).collect(),
                imports: Vec::new(),
                decorators: vec![component_decorator(&children)],
                header_comments: Vec::new(),
                body: std::mem::take(&mut definition.body),
            },
            range,
        });

        *node = Node::Fragment(FragmentNode {
            children: Vec::new(),
            range,
        });
        self.children
            .last_mut()
            .expect("parent component scope")
            .push(name);
        Ok(())
    }
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

fn parse_signature(
    definition: &crate::ast::DefinitionNode,
) -> Result<(String, TextRange, Vec<ParameterNode>, bool), CompileError> {
    let signature = definition.signature.trim_start();
    let (python, is_async) = if let Some(rest) = signature.strip_prefix("async component ") {
        (format!("async def {rest}"), true)
    } else if let Some(rest) = signature.strip_prefix("component ") {
        (format!("def {rest}"), false)
    } else {
        return Err(invalid_signature(definition.range));
    };

    let source = format!("{python}\n    pass");
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

        let (name_node, type_node, default_node, kind) = match node.kind() {
            "identifier" => (node, None, None, ParamKind::KeywordOnly),
            "typed_parameter" => {
                let type_node = node.child_by_field_name("type").expect("parameter type");
                let name_node = first_named_child_except(node, type_node).expect("parameter name");
                let kind = parameter_kind(name_node, definition, &name)?;
                (name_node, Some(type_node), None, kind)
            }
            "default_parameter" => (
                node.child_by_field_name("name").expect("parameter name"),
                None,
                Some(
                    node.child_by_field_name("value")
                        .expect("parameter default"),
                ),
                ParamKind::KeywordOnly,
            ),
            "typed_default_parameter" => (
                node.child_by_field_name("name").expect("parameter name"),
                Some(node.child_by_field_name("type").expect("parameter type")),
                Some(
                    node.child_by_field_name("value")
                        .expect("parameter default"),
                ),
                ParamKind::KeywordOnly,
            ),
            "dictionary_splat_pattern" => (node, None, None, ParamKind::VarKeyword),
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

        let kind = if name_node.kind() == "dictionary_splat_pattern" {
            ParamKind::VarKeyword
        } else {
            kind
        };
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

    Ok((name, name_range, params, is_async))
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
        "Add `*,` before the first prop:\n\n  component {name}(*, {parameters}):"
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
    .with_help("Use `component Name():` or `component Name(*, prop: Type):`.")
    .boxed()
    .into()
}

fn mapped_range(
    definition: &crate::ast::DefinitionNode,
    python_start: usize,
    python_end: usize,
) -> TextRange {
    const COMPONENT_TO_DEF_OFFSET: usize = 6;
    TextRange {
        start: position_at(
            definition.signature_range.start,
            &definition.signature,
            python_start + COMPONENT_TO_DEF_OFFSET,
        ),
        end: position_at(
            definition.signature_range.start,
            &definition.signature,
            python_end + COMPONENT_TO_DEF_OFFSET,
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
