//! Isolated printer proof. Production compilation still preserves source slices.

use ruff_python_ast::comparable::ComparableStmt;
use ruff_python_ast::visitor::source_order::{SourceOrderVisitor, TraversalSignal};
use ruff_python_ast::visitor::transformer::{self, Transformer};
use ruff_python_ast::{AnyNodeRef, ModModule, NodeKind};
use ruff_python_codegen::{Generator, Indentation};
use ruff_python_parser::parse_module;
use ruff_text_size::{Ranged, TextRange};

#[derive(Debug, PartialEq)]
enum MappingError {
    InvalidPython,
    ChangedStructure,
}

#[derive(Debug)]
// Nested byte ranges, not final IDE injection segments.
struct NodeMapping {
    source: TextRange,
    generated: TextRange,
}

fn print_and_map(module: &ModModule) -> Result<(String, Vec<NodeMapping>), MappingError> {
    let code = module
        .body
        .iter()
        .map(|stmt| Generator::new(&Indentation::default(), Default::default()).stmt(stmt))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed = parse_module(&code).map_err(|_| MappingError::InvalidPython)?;
    let mappings = map_node_ranges(module, parsed.syntax())?;
    Ok((code, mappings))
}

fn map_node_ranges(
    source: &ModModule,
    generated: &ModModule,
) -> Result<Vec<NodeMapping>, MappingError> {
    let source_nodes = collect_nodes(source);
    let generated_nodes = collect_nodes(generated);
    if source_nodes.len() != generated_nodes.len() {
        return Err(MappingError::ChangedStructure);
    }
    source_nodes
        .into_iter()
        .zip(generated_nodes)
        .map(
            |((kind, depth, source), (generated_kind, generated_depth, generated))| {
                if (kind, depth) != (generated_kind, generated_depth) {
                    return Err(MappingError::ChangedStructure);
                }
                Ok(NodeMapping { source, generated })
            },
        )
        .filter(|mapping| !matches!(mapping, Ok(mapping) if mapping.source.is_empty()))
        .collect()
}

fn collect_nodes(module: &ModModule) -> Vec<(NodeKind, usize, TextRange)> {
    #[derive(Default)]
    struct Nodes {
        entries: Vec<(NodeKind, usize, TextRange)>,
        depth: usize,
    }
    impl<'a> SourceOrderVisitor<'a> for Nodes {
        fn enter_node(&mut self, node: AnyNodeRef<'a>) -> TraversalSignal {
            // Ruff removes empty string parts created by escaped newlines.
            if !matches!(node, AnyNodeRef::InterpolatedStringLiteralElement(literal) if literal.value.is_empty())
            {
                self.entries.push((node.kind(), self.depth, node.range()));
            }
            self.depth += 1;
            TraversalSignal::Traverse
        }
        fn leave_node(&mut self, _: AnyNodeRef<'a>) {
            self.depth -= 1;
        }
    }
    let mut nodes = Nodes::default();
    nodes.visit_body(&module.body);
    nodes.entries
}

#[test]
fn recover_exact_ranges_after_spacing_changes() {
    let source = "result=title.upper()\n";
    let parsed = parse_module(source).unwrap();
    let (code, mappings) = print_and_map(parsed.syntax()).unwrap();
    assert_eq!(code, "result = title.upper()");
    let mapping = mappings
        .iter()
        .find(|m| &source[m.source] == "title.upper()")
        .unwrap();
    assert_eq!(mapping.source, TextRange::new(7.into(), 20.into()));
    assert_eq!(mapping.generated, TextRange::new(9.into(), 22.into()));
    assert_eq!(&code[mapping.generated], "title.upper()");
}

#[test]
fn distinguish_repeated_expressions_and_unicode_byte_offsets() {
    let source = "café=écho(雪)\nother=écho(雪)\n";
    let (code, mappings) = print_and_map(parse_module(source).unwrap().syntax()).unwrap();
    let calls: Vec<_> = mappings
        .iter()
        .filter(|m| &source[m.source] == "écho(雪)")
        .collect();
    assert_eq!(calls.len(), 2);
    for (mapping, (offset, _)) in calls.iter().zip(code.match_indices("écho(雪)")) {
        assert_eq!(usize::from(mapping.generated.start()), offset);
        assert_eq!(&code[mapping.generated], "écho(雪)");
    }
    assert_ne!(calls[0].source, calls[1].source);
}

#[test]
fn retain_user_ranges_inside_generated_wrappers() {
    let source = "title.upper()";
    let mut original = parse_module(source).unwrap().into_syntax();
    let ruff_python_ast::Stmt::Expr(original) = original.body.remove(0) else {
        panic!()
    };
    let mut wrapper = parse_module("render(None)").unwrap().into_syntax();
    let ruff_python_ast::Stmt::Expr(statement) = &mut wrapper.body[0] else {
        panic!()
    };
    statement.range = TextRange::default();
    let ruff_python_ast::Expr::Call(call) = statement.value.as_mut() else {
        panic!()
    };
    call.range = TextRange::default();
    call.arguments.range = TextRange::default();
    let ruff_python_ast::Expr::Name(name) = call.func.as_mut() else {
        panic!()
    };
    name.range = TextRange::default();
    call.arguments.args[0] = *original.value;
    let (code, mappings) = print_and_map(&wrapper).unwrap();
    assert_eq!(code, "render(title.upper())");
    assert!(!mappings.is_empty());
    for mapping in mappings {
        assert_eq!(&source[mapping.source], &code[mapping.generated]);
        assert!(mapping.generated.start() >= 7.into());
    }
}

#[test]
fn reject_different_node_counts_kinds_and_nesting() {
    for (source, generated) in [
        ("a()", "a(); b()"),
        ("a", "1"),
        ("f(g(x), y)", "f(g(x, y))"),
    ] {
        assert!(matches!(
            map_node_ranges(
                parse_module(source).unwrap().syntax(),
                parse_module(generated).unwrap().syntax()
            ),
            Err(MappingError::ChangedStructure)
        ));
    }
}

#[test]
fn recover_ranges_for_python_language_forms() {
    for source in [
        "",
        "x = ((a + b)) * c\n",
        "x = 'one' 'two'\ny = b'one' b'two'\n",
        "x = f'{name=} {value!r:>{width}}'\n",
        "x = t'Hello {name}'\n",
        "x = [f(y) for y in values if y]\n",
        "@decorate\nasync def f(x: int = 1, /, *, y=None) -> str:\n    return await g(x, y=y)\n",
        "if a:\n    f()\nelif b:\n    g()\nelse:\n    h()\n",
        "match x:\n    case {'a': [first, *rest]} if first:\n        pass\n",
        "try:\n    raise ValueError()\nexcept* ValueError as error:\n    pass\nfinally:\n    clean()\n",
    ] {
        assert_preserves_python(source);
    }
}

#[test]
fn recover_ranges_for_existing_python_fixtures() {
    let pattern = format!("{}/tests/**/*.expected.py", env!("CARGO_MANIFEST_DIR"));
    let paths: Vec<_> = glob::glob(&pattern).unwrap().map(Result::unwrap).collect();
    assert!(!paths.is_empty());
    for path in paths {
        let source = std::fs::read_to_string(&path).unwrap();
        assert_preserves_python(&source);
    }
}

fn assert_preserves_python(source: &str) {
    let mut original = parse_module(source).unwrap().into_syntax();
    let (code, mappings) = print_and_map(&original).unwrap_or_else(|e| panic!("{source}: {e:?}"));
    let mut generated = parse_module(&code).unwrap().into_syntax();
    // Empty literals have no runtime content, but Ruff's comparison retains them.
    struct RemoveEmptyLiterals;
    impl Transformer for RemoveEmptyLiterals {
        fn visit_f_string(&self, string: &mut ruff_python_ast::FString) {
            transformer::walk_f_string(self, string);
            string.elements = string.elements.iter()
                .filter(|element| !matches!(element, ruff_python_ast::InterpolatedStringElement::Literal(literal) if literal.value.is_empty()))
                .cloned().collect::<Vec<_>>().into();
        }
    }
    RemoveEmptyLiterals.visit_body(&mut original.body);
    RemoveEmptyLiterals.visit_body(&mut generated.body);
    assert_eq!(
        original
            .body
            .iter()
            .map(ComparableStmt::from)
            .collect::<Vec<_>>(),
        generated
            .body
            .iter()
            .map(ComparableStmt::from)
            .collect::<Vec<_>>(),
        "{source}"
    );
    for mapping in mappings {
        assert!(
            source
                .get(std::ops::Range::<usize>::from(mapping.source))
                .is_some()
        );
        assert!(
            code.get(std::ops::Range::<usize>::from(mapping.generated))
                .is_some()
        );
    }
}

#[test]
fn recover_expressions_after_empty_string_parts_disappear() {
    let source = "x = f'''\\\n{first}\\\n{second}'''";
    let (code, mappings) = print_and_map(parse_module(source).unwrap().syntax()).unwrap();
    assert_eq!(code, "x = f'''{first}{second}'''");
    for expression in ["first", "second"] {
        let mapping = mappings
            .iter()
            .find(|m| &source[m.source] == expression)
            .unwrap();
        assert_eq!(&code[mapping.generated], expression);
    }
    assert_preserves_python(source);
}

#[test]
fn expose_source_preservation_gap() {
    let source = "# keep this comment\nvalue = (\n    left  +  right  # and this one\n)\n";
    let (code, mappings) = print_and_map(parse_module(source).unwrap().syntax()).unwrap();
    assert_eq!(code, "value = left + right");
    let mapping = mappings
        .iter()
        .find(|m| &source[m.source] == "left  +  right")
        .unwrap();
    assert_eq!(&code[mapping.generated], "left + right");
}
