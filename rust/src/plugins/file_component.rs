//! Turn a rendering file into its implicit component. Files that only declare
//! names stay plain modules. This is file policy layered on the core: the
//! filename names the component; `---`, props, decorators, or markup select it.

use crate::ast::{Ast, Function, FunctionDefinition, Node, TextRange};

pub struct FileComponent;

impl FileComponent {
    /// `filename` is the stem the component is named after; defaults to `Render`.
    pub fn lower(ast: &mut Ast, filename: Option<&str>, has_separator: bool) {
        let module = &mut ast.module;
        let renders = has_separator
            || !module.params.is_empty()
            || !module.decorators.is_empty()
            || module.body.iter().any(renders_markup);
        if !renders {
            return;
        }

        let function = Function {
            is_async: false,
            params: std::mem::take(&mut module.params),
            imports: Vec::new(),
            decorators: std::mem::take(&mut module.decorators),
            header_comments: Vec::new(),
            body: std::mem::take(&mut module.body),
        };
        ast.file_component = Some(FunctionDefinition {
            name: filename
                .map(to_pascal_case)
                .unwrap_or_else(|| "Render".to_string()),
            name_range: TextRange::synthetic(),
            return_annotation: None,
            function,
            range: TextRange::synthetic(),
        });
    }
}

fn renders_markup(node: &Node) -> bool {
    match node {
        Node::Text(text) => !text.content.is_empty(),
        Node::Expression(_)
        | Node::Element(_)
        | Node::Component(_)
        | Node::Slot(_)
        | Node::If(_)
        | Node::For(_)
        | Node::Match(_)
        | Node::While(_)
        | Node::With(_)
        | Node::Try(_) => true,
        Node::Fragment(fragment) => fragment.children.iter().any(renders_markup),
        Node::Comment(_)
        | Node::Statement(_)
        | Node::Definition(_)
        | Node::Function(_)
        | Node::Import(_)
        | Node::Parameter(_)
        | Node::Decorator(_) => false,
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
