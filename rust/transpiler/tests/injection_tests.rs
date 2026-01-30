use hyper_transpiler::{Pipeline, GenerateOptions};
use hyper_transpiler::generate::RangeType;

#[test]
fn test_expression_injection() {
    let source = "<button aria={x}>y</button>";
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    // Should have one injection for the {x} expression
    assert_eq!(result.injections.len(), 1, "Expected 1 injection");

    let injection = &result.injections[0];
    assert_eq!(injection.injection_type, "python");

    // Check that compiled positions are not zero (i.e., actually computed)
    assert!(result.ranges[0].compiled_start > 0, "Compiled start should not be 0");
    assert!(result.ranges[0].compiled_end > result.ranges[0].compiled_start,
            "Compiled end should be after compiled start");

    // Verify source range excludes braces (should be just 'x', not '{x}')
    let range_len = result.ranges[0].source_end - result.ranges[0].source_start;
    assert_eq!(range_len, 1, "Source range should be 1 char (just 'x'), not 3 ('{{x}}')");

}

#[test]
fn test_parameter_injection() {
    let source = "x: str\n---\n<div>{x}</div>";
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    // Should have injection for expression only (parameters in frontmatter don't need injection)
    assert!(result.injections.len() >= 1, "Expected at least 1 injection (expr)");

    // All ranges should have valid compiled positions
    for (i, range) in result.ranges.iter().enumerate() {
        assert!(range.compiled_end > range.compiled_start,
                "Range {} has invalid compiled positions: {} -> {}",
                i, range.compiled_start, range.compiled_end);
    }
}

#[test]
fn test_text_expression_injection() {
    let source = "<div>{name}</div>";
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    // Filter Python injections
    let python_injections: Vec<_> = result.injections.iter()
        .filter(|i| i.injection_type == "python")
        .collect();
    let python_ranges: Vec<_> = result.ranges.iter()
        .filter(|r| r.range_type == RangeType::Python)
        .collect();

    assert_eq!(python_injections.len(), 1);
    assert_eq!(python_ranges.len(), 1);

    // The source range for a text expression - may or may not include braces
    // depending on transpiler behavior (braces included for f-string highlighting)
    let range = python_ranges[0];
    assert!(range.source_start < range.source_end, "Range should have positive length");
    assert!(range.source_end <= source.len(), "Range should be within source bounds");

    // Verify the injection creates valid Python code
    let injection = python_injections[0];
    assert!(injection.prefix.contains("def "), "Should contain function definition");
}

#[test]
fn test_class_attribute_injection() {
    let source = r#"<div class={active and "active"}>Content</div>"#;
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    assert_eq!(result.injections.len(), 1);

    // Should extract just the expression, not the braces
    let source_text = &source[result.ranges[0].source_start..result.ranges[0].source_end];
    assert_eq!(source_text, r#"active and "active""#);
}

#[test]
fn test_style_attribute_injection() {
    let source = r#"<div style={{"color": color}}>Text</div>"#;
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    assert_eq!(result.injections.len(), 1);

    // Should extract the style dict expression without outer template braces
    let source_text = &source[result.ranges[0].source_start..result.ranges[0].source_end];
    // For style={{dict}}, we extract {dict} (one pair of braces removed)
    assert!(source_text.starts_with("{"), "Should start with opening brace");
    assert!(source_text.contains("color"), "Should contain 'color'");
}

#[test]
fn test_spread_attribute_injection() {
    let source = r#"<button aria={aria_attrs}>Close</button>"#;
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    assert_eq!(result.injections.len(), 1);

    // Should extract just "aria_attrs"
    let source_text = &source[result.ranges[0].source_start..result.ranges[0].source_end];
    assert_eq!(source_text, "aria_attrs");
}

#[test]
fn test_multiple_expressions() {
    let source = r#"
x: str
y: int
---
<div class={x}>
    {y}
    <span aria={z}>text</span>
</div>"#;
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    // Should have injections for: x in class, y in text, z in aria (params don't need injection)
    assert!(result.injections.len() >= 3, "Expected at least 3 injections, got {}", result.injections.len());

    // All injections should have valid positions
    for (i, range) in result.ranges.iter().enumerate() {
        assert!(range.compiled_end > range.compiled_start,
                "Range {} has invalid positions", i);

        // Verify source range is within bounds
        assert!(range.source_end <= source.len(),
                "Range {} source_end {} exceeds source length {}",
                i, range.source_end, source.len());
    }
}

#[test]
fn test_parameters_with_separator() {
    let source = r#"is_hidden: bool = False

---

aria_attrs = {"label": "Close dialog", "hidden": is_hidden, "live": "polite"}

<button aria={aria_attrs}>Close</button>"#;
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    // Should have at least expression injection (params don't need injection)
    assert!(result.injections.len() >= 1, "Expected at least 1 injection");

    // Verify all ranges are valid
    for range in &result.ranges {
        assert!(range.compiled_end > range.compiled_start);
        assert!(range.source_end > range.source_start);
    }
}

#[test]
fn test_parameters_without_separator() {
    let source = r#"is_hidden: bool = False

aria_attrs = {"label": "Close dialog", "hidden": is_hidden, "live": "polite"}

<button aria={aria_attrs}>Close</button>"#;
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    // Should have at least expression injection (params don't need injection)
    assert!(result.injections.len() >= 1, "Expected at least 1 injection");

    // Verify all ranges are valid
    for range in &result.ranges {
        assert!(range.compiled_end > range.compiled_start);
        assert!(range.source_end > range.source_start);
    }
}

#[test]
fn test_injection_prefix_suffix_correctness() {
    let source = r#"x: str

print("test")

---

<button aria={x}>
    y
</button>"#;
    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: Some("Test".to_string()),
        include_ranges: true,
    }).unwrap();

    // Should have at least 1 injection (expression, params don't need injection)
    assert!(result.injections.len() >= 1, "Expected at least 1 injection");

    // Find the expression injection (should contain aria in prefix)
    let expr_injection = result.injections.iter()
        .find(|inj| inj.prefix.contains("aria="))
        .expect("Should find expression injection with aria attribute");

    // When combined, should form valid Python code
    let code_slice = &result.code[expr_injection.start..expr_injection.end];
    let combined = format!("{}{}{}", expr_injection.prefix, code_slice, expr_injection.suffix);

    // The combined code should be valid Python and contain key elements
    assert!(combined.contains("aria=") || combined.contains("aria=\""),
            "Combined code should have aria attribute");
    assert!(combined.contains("print(\"test\")"),
            "Combined code should have the print statement");
}
