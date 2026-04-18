mod common;

use common::compile;

#[test]
fn test_selective_helper_imports() {
    // Template that uses class attribute with dynamic expression
    let source = r#"<div class={active and "active"}>Hello</div>"#;

    let code = compile(source);

    //Should import html and render_class for class attributes
    assert!(code.contains("from hyper import html, render_class"));
    assert!(code.contains("render_class("));
    // Should NOT contain old markers
    assert!(!code.contains("replace_markers"));
    assert!(!code.contains('‹'));
}

#[test]
fn test_async_detection() {
    // Template with async for
    let source = r#"items: list
---
async for item in items:
    <li>{item}</li>
end"#;

    let code = compile(source);

    //Should have async def
    assert!(code.contains("async def Render"));
}

#[test]
fn test_non_async_template() {
    // Template without await/async
    let source = r#"<div>Hello</div>"#;

    let code = compile(source);

    //Should NOT have async def
    assert!(!code.contains("async def"));
    assert!(code.contains("def Render():"));

    // Should have component import
    assert!(code.contains("from hyper import html"));
    assert!(code.contains("@html"));
}

#[test]
fn test_content_slot_parameter() {
    // Template with only default slot (no explicit parameters)
    let source = r#"<div>{...}</div>"#;

    let code = compile(source);

    //Function should have _content parameter for default slot
    assert!(code.contains("_content"));

    // Should be optional with Iterable type
    assert!(code.contains("_content: Iterable[str] | None = None"));
}

#[test]
fn test_multiple_helpers() {
    // Template that uses multiple attribute helpers
    let source = r#"<div class={cls} style={{"color": "red"}}>Hello</div>"#;

    let code = compile(source);

    //Should import render_class and render_style
    assert!(code.contains("render_class"));
    assert!(code.contains("render_style"));
    // Should NOT contain old markers
    assert!(!code.contains("replace_markers"));
    assert!(!code.contains('‹'));
}

#[test]
fn test_component_always_imported() {
    // Template with static HTML only (no expressions, no markers)
    let source = r#"<div>Hello World</div>"#;

    let code = compile(source);

    //Should always import html for the decorator
    assert!(code.contains("from hyper import html"));
    assert!(code.contains("@html"));

    // Should NOT have replace_markers (no markers needed)
    assert!(!code.contains("replace_markers"));
}

#[test]
fn test_parameters_with_slots() {
    // Template with parameters and default slot
    let source = r#"title: str
---
<div>
    <h1>{title}</h1>
    {...}
</div>"#;

    let code = compile(source);

    //Should have parameter in signature (keyword-only)
    assert!(code.contains("*, title: str"));

    // Should have _content parameter for default slot
    assert!(code.contains("_content: Iterable[str] | None = None"));
}

#[test]
fn test_async_with_parameters() {
    // Async template with parameters
    let source = r#"url: str
items: list
---
async for item in items:
    <li>{item}</li>
end"#;

    let code = compile(source);

    assert!(code.contains("async def Render"));
    assert!(code.contains("url: str"));
    assert!(code.contains("items: list"));
}
