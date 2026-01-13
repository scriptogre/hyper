use hyper_transpiler::{Pipeline, GenerateOptions};

#[test]
fn test_selective_helper_imports() {
    // Template that uses class marker
    let source = r#"<div class={active and "active"}>Hello</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should import replace_markers for class markers
    assert!(result.code.contains("from hyper import replace_markers"));
    assert!(result.code.contains("‹CLASS:"));

    // Should not import escape (no expression escaping needed)
    assert!(!result.code.contains("import escape"));
}

#[test]
fn test_async_detection() {
    // Template with await
    let source = r#"url: str
---
<div>{await fetch(url)}</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should have async import line
    assert!(result.code.contains("async from hyper import"));

    // Should import escape for expression
    assert!(result.code.contains("import escape"));
}

#[test]
fn test_non_async_template() {
    // Template without await
    let source = r#"<div>Hello</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should NOT have async import
    assert!(!result.code.contains("async from"));
    assert!(result.code.contains("def Render() -> str:"));

    // Should have no imports at all
    assert!(!result.code.contains("from hyper import"));
}

#[test]
fn test_children_slot_parameter() {
    // Template with only default slot (no explicit parameters)
    let source = r#"<div>{...}</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Function should have _children parameter (actual name may vary based on slot naming)
    assert!(result.code.contains("_children"));

    // Should be optional with default empty string
    assert!(result.code.contains("_children: str = \"\""));
}

#[test]
fn test_multiple_markers() {
    // Template that uses multiple markers
    let source = r#"<div class={cls} style={{"color": "red"}}>Hello</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should import replace_markers for both class and style
    assert!(result.code.contains("from hyper import replace_markers"));
    assert!(result.code.contains("‹CLASS:"));
    assert!(result.code.contains("‹STYLE:"));
}

#[test]
fn test_no_extra_helpers_when_not_needed() {
    // Template with static HTML only (no expressions, no markers)
    let source = r#"<div>Hello World</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    // Should not import anything (no escape, no replace_markers needed)
    assert!(!result.code.contains("from hyper import"));
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

    // Should have parameter in signature
    assert!(result.code.contains("title: str"));

    // Should have _children parameter for default slot with correct naming
    // (Note: slot parameters end with _children suffix)
    assert!(result.code.contains("_children"));
}

#[test]
fn test_async_with_parameters() {
    // Async template with parameters
    let source = r#"url: str
items: list
---
<div>{await fetch(url)}</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();

    assert!(result.code.contains("async from hyper import"));
    assert!(result.code.contains("url: str"));
    assert!(result.code.contains("items: list"));
}
