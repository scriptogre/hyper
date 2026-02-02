use hyper_transpiler::{GenerateOptions, Pipeline};

fn compile_v2(source: &str) -> String {
    let mut pipeline = Pipeline::standard();
    let options = GenerateOptions::default();
    pipeline.compile(source, &options).unwrap().code
}

#[test]
fn test_simple_element() {
    let source = r#"<div>Hello World</div>"#;
    let output = compile_v2(source);

    println!("Output:\n{}", output);
    assert!(output.contains("@component"));
    assert!(output.contains("def Render():"));
    assert!(output.contains("yield \"\"\"<div>"));
}

#[test]
fn test_expression() {
    let source = r#"<div>{name}</div>"#;
    let output = compile_v2(source);

    // Should use ESCAPE marker instead of escape() function
    assert!(output.contains("‹ESCAPE:{name}›"));
}

#[test]
fn test_parameters() {
    let source = r#"name: str
---
<div>{name}</div>"#;

    let output = compile_v2(source);

    println!("Output:\n{}", output);
    assert!(output.contains("def Render(*, name: str):"));
    assert!(output.contains("@component"));
}

#[test]
fn test_for_loop() {
    let source = r#"items: list
---
for item in items:
    <li>{item}</li>
end"#;

    let output = compile_v2(source);

    println!("Output:\n{}", output);
    assert!(output.contains("def Render(*, items: list):"));
    assert!(output.contains("for item in items:"));
    assert!(!output.contains("for item in items::")); // No double colon
}

#[test]
fn test_if_statement() {
    let source = r#"show: bool
---
if show:
    <p>Visible</p>
else:
    <p>Hidden</p>
end"#;

    let output = compile_v2(source);

    assert!(output.contains("if show:"));
    assert!(output.contains("else:"));
}

#[test]
fn test_nested_elements() {
    let source = r#"<div><span>Hello</span></div>"#;
    let output = compile_v2(source);

    assert!(output.contains("<div>"));
    assert!(output.contains("<span>"));
    assert!(output.contains("</span>"));
    assert!(output.contains("</div>"));
}

#[test]
fn test_multiple_parameters() {
    let source = r#"name: str
age: int
active: bool = True
---
<div>{name} is {age}</div>"#;

    let output = compile_v2(source);

    println!("Output:\n{}", output);
    assert!(output.contains("def Render("));
    assert!(output.contains("name: str"));
    assert!(output.contains("age: int"));
}
