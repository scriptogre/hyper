use hyper_transpiler::{Pipeline, GenerateOptions};

#[test]
fn test_ranges_with_metadata() {
    // Template with async, slots, and helpers
    let source = r#"url: str
---
<div class={active}>
    {await fetch(url)}
    {...}
</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: None,
        include_ranges: true,
    }).unwrap();

    println!("=== Generated Code ===");
    println!("{}", result.code);
    println!("\n=== Ranges ===");
    for range in &result.ranges {
        println!("{:?}", range);
    }
    println!("==================");

    // Verify ranges are still being tracked
    assert!(!result.ranges.is_empty(), "Ranges should be tracked for IDE integration");
}

#[test]
fn test_ranges_track_expressions() {
    let source = r#"name: str
---
<div>{name}</div>"#;

    let mut pipeline = Pipeline::standard();
    let result = pipeline.compile(source, &GenerateOptions {
        function_name: None,
        include_ranges: true,
    }).unwrap();

    println!("\n=== Code ===");
    println!("{}", result.code);
    println!("\n=== Ranges ===");
    for range in &result.ranges {
        println!("Source [{}-{}] -> Compiled [{}-{}]",
            range.source_start, range.source_end,
            range.compiled_start, range.compiled_end);
    }

    // Should have ranges for both parameter and expression
    assert!(result.ranges.len() >= 1, "Should have at least one range");

    // Check that at least one range has valid positions
    let has_valid_range = result.ranges.iter().any(|r| {
        r.source_end > r.source_start && r.compiled_end > r.compiled_start
    });
    assert!(has_valid_range, "Should have at least one valid range with proper positions");
}
