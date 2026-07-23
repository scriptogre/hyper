use hyper::{CompileOptions, compile};

#[test]
fn explicit_component_is_hoisted_and_attached() {
    let source = r#"title: str
---
component Header(*, title: str):
    <header>{title}</header>
end

<{Header} title={title} />
"#;
    let result = compile(
        source,
        &CompileOptions {
            function_name: Some("Page".to_string()),
            include_ranges: false,
        },
    )
    .expect("component should compile");

    let header = result
        .code
        .find("def Header(")
        .expect("declared component should be generated");
    let page = result
        .code
        .find("def Page(")
        .expect("page should be generated");

    assert!(header < page, "declared component must be hoisted");
    assert!(result.code.contains("@component\ndef Header("));
    assert!(
        result
            .code
            .contains("@component(subcomponents=[Header])\ndef Page(")
    );
    assert!(
        result
            .code
            .contains("yield from Header.stream(title=title)")
    );
}

#[test]
fn component_signature_supports_multiline_props_and_attrs() {
    let code = compile_code(
        r#"component Button(
    *,
    label: str,
    kind: str = "button",
    **attrs,
):
    <button {**attrs}>{label}</button>
end
"#,
    );

    assert!(code.contains("def Button(\n        *,\n        label: str,"));
    assert!(code.contains("kind: str = \"button\","));
    assert!(code.contains("**attrs,"));
}

#[test]
fn slots_are_scoped_to_the_declared_component() {
    let code = compile_code(
        r#"title: str
---
component Layout(*, title: str):
    <header>{...header}</header>
    <main>{...}</main>
end

<{Layout} title={title} />
"#,
    );
    let layout = code.find("def Layout(").expect("layout definition");
    let page = code.find("def Page(").expect("page definition");
    let layout_code = &code[layout..page];
    let page_code = &code[page..];

    assert!(layout_code.contains("content: Iterable[str] | None = None,"));
    assert!(layout_code.contains("header: Iterable[str] | None = None,"));
    assert!(!page_code.contains("content: Iterable[str]"));
    assert!(!page_code.contains("header: Iterable[str]"));
}

#[test]
fn component_call_binds_single_element_named_slot() {
    let code = compile_code(
        r#"from app.components import Card
---
<{Card} title="Delete item">
    <p>This cannot be undone.</p>
    <button {...actions}>Delete</button>
</{Card}>
"#,
    );
    let page = code.find("def Page(").expect("page definition");
    let page_code = &code[page..];

    assert!(page_code.contains("def _card_content():"));
    assert!(page_code.contains("def _card_actions():"));
    assert!(page_code.contains("yield \"\"\"<button>Delete</button>\"\"\""));
    assert!(page_code.contains("actions=_card_actions()"));
    assert!(!page_code.contains("slot:actions"));
    assert!(!page_code.contains("actions: Iterable[str]"));
}

#[test]
fn component_call_binds_explicit_named_slot_wrapper() {
    let code = compile_code(
        r#"from app.components import Card
---
<{Card} title="Delete item">
    <p>This cannot be undone.</p>
    <{...actions}>
        <button>Delete</button>
    </{...actions}>
</{Card}>
"#,
    );
    let page = code.find("def Page(").expect("page definition");
    let page_code = &code[page..];

    assert!(page_code.contains("def _card_content():"));
    assert!(page_code.contains("def _card_actions():"));
    assert!(page_code.contains("yield \"\"\"<button>Delete</button>\"\"\""));
    assert!(page_code.contains("actions=_card_actions()"));
    assert!(!page_code.contains("actions: Iterable[str]"));
    assert!(!page_code.contains("if actions is not None:"));
}

#[test]
fn named_slot_binding_preserves_component_namespaces() {
    let code = compile_code(
        r#"---
<{UI.Card}>
    <button {...actions}>Save</button>
</{UI.Card}>
"#,
    );

    assert!(code.contains("def _u_i_card_actions():"));
    assert!(code.contains("yield from UI.Card.stream("));
    assert!(code.contains("actions=_u_i_card_actions()"));
}

#[test]
fn duplicate_named_slot_fills_are_rejected() {
    let source = r#"---
<{Card}>
    <button {...actions}>Save</button>
    <button {...actions}>Cancel</button>
</{Card}>
"#;
    let error = compile(source, &CompileOptions::default())
        .expect_err("duplicate named slot fills should fail");
    let message = error.render(source, "Page.hyper");

    assert!(message.contains("`actions` slot is filled more than once"));
    assert!(message.contains("first fill"));
}

#[test]
fn nested_components_attach_to_their_direct_parent() {
    let code = compile_code(
        r#"---
component Header():
    component Logo():
        <strong>Hyper</strong>
    end
    <{Logo} />
end

<{Header} />
"#,
    );

    assert!(code.contains("@component\ndef Logo("));
    assert!(code.contains("@component(subcomponents=[Logo])\ndef Header("));
    assert!(code.contains("@component(subcomponents=[Header])\ndef Page("));
}

#[test]
fn child_async_detection_does_not_change_the_parent() {
    let code = compile_code(
        r#"---
async component Results(*, query: str):
    rows = await search(query)
    <p>{rows}</p>
end
"#,
    );

    assert!(code.contains("async def Results("));
    assert!(code.contains("\ndef Page():"));
    assert!(!code.contains("async def Page("));
}

#[test]
fn component_props_require_the_keyword_only_marker() {
    let source = "component Button(label: str):\n    <button>{label}</button>\nend\n";
    let error = compile(source, &CompileOptions::default())
        .expect_err("positional component prop should fail");
    let message = error.render(source, "Button.hyper");

    assert!(message.contains("keyword-only"));
    assert!(message.contains("component Button(*, label: str):"));
}

fn compile_code(source: &str) -> String {
    compile(
        source,
        &CompileOptions {
            function_name: Some("Page".to_string()),
            include_ranges: false,
        },
    )
    .expect("component should compile")
    .code
}
