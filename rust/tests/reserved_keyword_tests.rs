//! The Python keyword `class` used as an identifier (param, kwarg, or expression)
//! must compile to `class_`. Builtins like `type` are valid identifiers and must
//! be left untouched, so `type(x)` keeps calling the builtin.

mod common;

use common::compile;

#[test]
fn test_reserved_keyword_param_renamed_in_signature() {
    let py = compile("class: str = \"\"\ntype: str = \"button\"\n---\n<div>{class} {type}</div>\n");

    assert!(
        py.contains("class_: str"),
        "param `class` must compile to `class_` (it is a Python keyword):\n{py}"
    );
    assert!(
        py.contains("type: str") && !py.contains("type_"),
        "param `type` is a valid identifier and must stay `type`:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_component_call_kwarg_renamed() {
    let py = compile("<{Dropdown} class=\"btn\" type=\"submit\" />\n");

    assert!(
        py.contains("class_=\"btn\""),
        "component-call kwarg `class` must compile to `class_`:\n{py}"
    );
    assert!(
        py.contains("type=\"submit\"") && !py.contains("type_"),
        "component-call kwarg `type` is a valid identifier and must stay `type`:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_component_call_expression_kwarg_renamed() {
    let py = compile("<{Dropdown} class={x} />\n");

    assert!(
        py.contains("class_=x") || py.contains("class_= x"),
        "component-call expression kwarg `class` must compile to `class_`:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_text_expression_renamed() {
    let py = compile("class: str = \"\"\n---\n<p>{class}</p>\n");

    assert!(
        py.contains("escape(class_)"),
        "text-position {{class}} must reference the renamed param `class_`:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_buried_in_call_is_renamed() {
    let py = compile("class: str = \"\"\n---\n<div>{func(class)}</div>\n");

    assert!(
        py.contains("func(class_)"),
        "a reserved keyword buried in a call must be renamed:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_buried_in_collection_is_renamed() {
    let py = compile("class: str = \"\"\n---\n<div>{[class, 1]}</div>\n");

    assert!(
        py.contains("[class_, 1]"),
        "a reserved keyword buried in a list must be renamed:\n{py}"
    );
}

#[test]
fn test_string_literal_class_is_not_renamed() {
    let py = compile("x: str = \"\"\n---\n<div>{x == \"class\"}</div>\n");

    assert!(
        py.contains("\"class\""),
        "the string \"class\" must survive:\n{py}"
    );
    assert!(
        !py.contains("class_"),
        "a string literal must never be renamed:\n{py}"
    );
}

#[test]
fn test_string_and_identifier_mixed() {
    let py = compile("class: str = \"\"\n---\n<div>{[\"class\", class]}</div>\n");

    assert!(
        py.contains("[\"class\", class_]"),
        "string stays literal, identifier renames:\n{py}"
    );
}

#[test]
fn test_builtin_type_call_is_not_renamed() {
    let py = compile("x: object = None\n---\n<div>{type(x)}</div>\n");

    assert!(
        py.contains("type(x)"),
        "the builtin `type()` must keep working:\n{py}"
    );
    assert!(
        !py.contains("type_"),
        "a builtin must never be renamed:\n{py}"
    );
}

#[test]
fn test_attribute_access_is_not_renamed() {
    let py = compile("obj: object = None\n---\n<div>{obj.classname}</div>\n");

    assert!(
        py.contains("obj.classname"),
        "attribute access must not rename:\n{py}"
    );
    assert!(
        !py.contains("classname_"),
        "attribute access was wrongly renamed:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_in_if_condition_is_renamed() {
    let py = compile("class: bool = False\n---\nif class:\n    <p>x</p>\nend\n");
    assert!(py.contains("if class_:"), "if condition must rename:\n{py}");
}

#[test]
fn test_reserved_keyword_in_elif_condition_is_renamed() {
    let py = compile(
        "class: bool = False\nn: int = 0\n---\nif n:\n    <p>a</p>\nelif class:\n    <p>b</p>\nend\n",
    );
    assert!(
        py.contains("elif class_:"),
        "elif condition must rename:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_in_for_iterable_is_renamed() {
    let py = compile("class: list = []\n---\nfor x in class:\n    <p>{x}</p>\nend\n");
    assert!(
        py.contains("for x in class_:"),
        "for iterable must rename:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_in_while_condition_is_renamed() {
    let py = compile("class: int = 0\n---\nwhile class:\n    <p>x</p>\nend\n");
    assert!(
        py.contains("while class_:"),
        "while condition must rename:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_in_with_items_is_renamed() {
    let py = compile("class: object = None\n---\nwith class as c:\n    <p>{c}</p>\nend\n");
    assert!(
        py.contains("with class_ as c:"),
        "with items must rename:\n{py}"
    );
}

#[test]
fn test_reserved_keyword_in_match_subject_is_renamed() {
    let py = compile("class: int = 0\n---\nmatch class:\n    case 1:\n        <p>x</p>\nend\n");
    assert!(
        py.contains("match class_:"),
        "match subject must rename:\n{py}"
    );
}

#[test]
fn test_class_definition_statement_is_not_renamed() {
    let py = compile("class Foo:\n    pass\n---\n<div>hi</div>\n");

    assert!(
        py.contains("class Foo:"),
        "a real `class Foo:` definition must stay valid Python:\n{py}"
    );
    assert!(
        !py.contains("class_ Foo"),
        "a class definition must not be renamed like an identifier:\n{py}"
    );
}
