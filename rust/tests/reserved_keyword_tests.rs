//! Reserved Python keywords (`class`, `type`) used as component props or
//! component-call attributes must compile to safe names (`class_`, `type_`),
//! matching the rename already applied to body expressions.

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
        py.contains("type_: str"),
        "param `type` must compile to `type_` (it is a Python keyword):\n{py}"
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
        py.contains("type_=\"submit\""),
        "component-call kwarg `type` must compile to `type_`:\n{py}"
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
