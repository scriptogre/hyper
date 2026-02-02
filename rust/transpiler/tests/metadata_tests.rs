use hyper_transpiler::{Pipeline, GenerateOptions};

#[test]
fn test_selective_helper_imports() {
    // Template that uses class marker
    let source = r#"<div class={active and "active"}>Hello</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should import component and replace_markers for class markers
    assert!(result.code.contains("from hyper import component, replace_markers"));
    assert!(result.code.contains("‹CLASS:"));
}

#[test]
fn test_async_detection() {
    // Template with async for
    let source = r#"items: list
---
async for item in items:
    <li>{item}</li>
end"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should have async def
    assert!(result.code.contains("async def Render"));
}

#[test]
fn test_non_async_template() {
    // Template without await/async
    let source = r#"<div>Hello</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should NOT have async def
    assert!(!result.code.contains("async def"));
    assert!(result.code.contains("def Render():"));

    // Should have component import
    assert!(result.code.contains("from hyper import component"));
    assert!(result.code.contains("@component"));
}

#[test]
fn test_content_slot_parameter() {
    // Template with only default slot (no explicit parameters)
    let source = r#"<div>{...}</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Function should have _content parameter for default slot
    assert!(result.code.contains("_content"));

    // Should be optional with Iterable type
    assert!(result.code.contains("_content: Iterable[str] | None = None"));
}

#[test]
fn test_multiple_markers() {
    // Template that uses multiple markers
    let source = r#"<div class={cls} style={{"color": "red"}}>Hello</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should import replace_markers for both class and style
    assert!(result.code.contains("replace_markers"));
    assert!(result.code.contains("‹CLASS:"));
    assert!(result.code.contains("‹STYLE:"));
}

#[test]
fn test_component_always_imported() {
    // Template with static HTML only (no expressions, no markers)
    let source = r#"<div>Hello World</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should always import component for the decorator
    assert!(result.code.contains("from hyper import component"));
    assert!(result.code.contains("@component"));

    // Should NOT have replace_markers (no markers needed)
    assert!(!result.code.contains("replace_markers"));
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

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should have parameter in signature (keyword-only)
    assert!(result.code.contains("*, title: str"));

    // Should have _content parameter for default slot
    assert!(result.code.contains("_content: Iterable[str] | None = None"));
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

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    assert!(result.code.contains("async def Render"));
    assert!(result.code.contains("url: str"));
    assert!(result.code.contains("items: list"));
}
