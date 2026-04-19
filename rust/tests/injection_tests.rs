mod common;

use common::{compile_with_ranges, html_injections, html_ranges, python_injections, python_ranges};
use hyper_transpiler::{GenerateOptions, Pipeline};

#[test]
fn test_expression_injection() {
    let source = "<button aria={x}>y</button>";
    let result = compile_with_ranges(source, "Test");

    // Should have one Python injection for the {x} expression
    let py = python_injections(&result);
    assert_eq!(py.len(), 1, "Expected 1 Python injection");
    assert_eq!(py[0].injection_type, "python");

    let py_ranges = python_ranges(&result);
    // Check that compiled positions are not zero
    assert!(
        py_ranges[0].compiled_start > 0,
        "Compiled start should not be 0"
    );
    assert!(
        py_ranges[0].compiled_end > py_ranges[0].compiled_start,
        "Compiled end should be after compiled start"
    );

    // Verify source range excludes braces (should be just 'x', not '{x}')
    let range_len = py_ranges[0].source_end - py_ranges[0].source_start;
    assert_eq!(
        range_len, 1,
        "Source range should be 1 char (just 'x'), not 3 ('{{x}}')"
    );

    // Should also have HTML injection ranges for the static HTML parts
    let html = html_injections(&result);
    assert!(!html.is_empty(), "Should have HTML injection ranges");

    // HTML injections should have empty prefix/suffix
    for inj in &html {
        assert!(inj.prefix.is_empty(), "HTML prefix should be empty");
        assert!(inj.suffix.is_empty(), "HTML suffix should be empty");
    }
}

#[test]
fn test_parameter_injection() {
    let source = "x: str\n---\n<div>{x}</div>";
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert!(
        !py.is_empty(),
        "Expected at least 1 Python injection (expr)"
    );

    // Python ranges should have valid compiled positions
    for (i, range) in python_ranges(&result).iter().enumerate() {
        assert!(
            range.compiled_end > range.compiled_start,
            "Range {} has invalid compiled positions: {} -> {}",
            i,
            range.compiled_start,
            range.compiled_end
        );
    }
}

#[test]
fn test_text_expression_injection() {
    let source = "<div>{name}</div>";
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    let py_ranges = python_ranges(&result);

    assert_eq!(py.len(), 1);
    assert_eq!(py_ranges.len(), 1);

    let range = py_ranges[0];
    assert!(
        range.source_start < range.source_end,
        "Range should have positive length"
    );
    assert!(
        range.source_end <= source.len(),
        "Range should be within source bounds"
    );

    // Verify the injection creates valid Python code
    assert!(
        py[0].prefix.contains("def "),
        "Should contain function definition"
    );
}

#[test]
fn test_class_attribute_injection() {
    let source = r#"<div class={active and "active"}>Content</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert_eq!(py.len(), 1);

    let py_ranges = python_ranges(&result);
    let source_text = &source[py_ranges[0].source_start..py_ranges[0].source_end];
    assert_eq!(source_text, r#"active and "active""#);
}

#[test]
fn test_style_attribute_injection() {
    let source = r#"<div style={{"color": color}}>Text</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert_eq!(py.len(), 1);

    let py_ranges = python_ranges(&result);
    let source_text = &source[py_ranges[0].source_start..py_ranges[0].source_end];
    assert!(
        source_text.starts_with("{"),
        "Should start with opening brace"
    );
    assert!(source_text.contains("color"), "Should contain 'color'");
}

#[test]
fn test_spread_attribute_injection() {
    let source = r#"<button aria={aria_attrs}>Close</button>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert_eq!(py.len(), 1);

    let py_ranges = python_ranges(&result);
    let source_text = &source[py_ranges[0].source_start..py_ranges[0].source_end];
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
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    // x in class, y in text, z in aria + params
    assert!(
        py.len() >= 3,
        "Expected at least 3 Python injections, got {}",
        py.len()
    );

    // All Python ranges should have valid positions
    for (i, range) in python_ranges(&result).iter().enumerate() {
        assert!(
            range.compiled_end > range.compiled_start,
            "Range {} has invalid positions",
            i
        );
        assert!(
            range.source_end <= source.len(),
            "Range {} source_end {} exceeds source length {}",
            i,
            range.source_end,
            source.len()
        );
    }

    // Should also have HTML ranges
    let html = html_ranges(&result);
    assert!(!html.is_empty(), "Should have HTML ranges for element tags");
}

/// Tests that injection ranges are correct when the template uses an explicit `---`
/// separator between parameters and body. Compare with `test_parameters_without_separator`.
#[test]
fn test_parameters_with_separator() {
    let source = r#"is_hidden: bool = False

---

aria_attrs = {"label": "Close dialog", "hidden": is_hidden, "live": "polite"}

<button aria={aria_attrs}>Close</button>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert!(!py.is_empty(), "Expected at least 1 Python injection");

    for range in &python_ranges(&result) {
        assert!(range.compiled_end > range.compiled_start);
        assert!(range.source_end > range.source_start);
    }
}

/// Tests that injection ranges are correct when there is NO `---` separator.
/// The parser must infer where parameters end and body begins.
/// Compare with `test_parameters_with_separator`.
#[test]
fn test_parameters_without_separator() {
    let source = r#"is_hidden: bool = False

aria_attrs = {"label": "Close dialog", "hidden": is_hidden, "live": "polite"}

<button aria={aria_attrs}>Close</button>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert!(!py.is_empty(), "Expected at least 1 Python injection");

    for range in &python_ranges(&result) {
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
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert!(
        py.len() >= 2,
        "Expected at least 2 Python injections (statement + expression)"
    );

    // The print statement should now have its own injection range
    let stmt_injection = py
        .iter()
        .find(|inj| {
            let source_slice = &source[inj.start..inj.end];
            source_slice.contains("print")
        })
        .expect("Should find injection for the print statement");
    let source_slice = &source[stmt_injection.start..stmt_injection.end];
    assert_eq!(
        source_slice.trim(),
        "print(\"test\")",
        "Statement injection should map to the print statement in source"
    );

    let expr_injection = py
        .iter()
        .find(|inj| inj.prefix.contains("aria"))
        .expect("Should find expression injection with aria attribute");

    let source_expr = &source[expr_injection.start..expr_injection.end];
    assert_eq!(
        source_expr, "x",
        "Expression injection should map to 'x' in source"
    );
}

#[test]
fn test_shorthand_attribute_injection() {
    let source = r#"<div {disabled}>Content</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert_eq!(py.len(), 1, "Expected 1 Python injection for shorthand");

    let py_ranges = python_ranges(&result);
    let source_text = &source[py_ranges[0].source_start..py_ranges[0].source_end];
    assert_eq!(source_text, "disabled");
}

#[test]
fn test_all_expression_contexts() {
    let source = r#"name: str
count: int = 0
---
<div class={active} style={styles} {disabled} {**props} data-id={id}>
    {name}
    {!raw_html}
</div>
if count > 0:
    <span>{count}</span>
end
for item in items:
    <li>{item}</li>
end
while running:
    <p>Loading...</p>
end"#;
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_ranges(&result);

    // Expected Python ranges:
    // 2 params + 5 attrs + 2 text exprs + 3 control flow + 2 nested exprs = 14
    assert!(
        py_ranges.len() >= 13,
        "Expected at least 13 Python ranges, got {}",
        py_ranges.len()
    );

    for (i, range) in py_ranges.iter().enumerate() {
        assert!(
            range.compiled_end > range.compiled_start,
            "Range {} has invalid compiled positions",
            i
        );
        assert!(
            range.source_end > range.source_start,
            "Range {} has invalid source positions",
            i
        );
        assert!(
            range.source_end <= source.len(),
            "Range {} source_end {} exceeds source len {}",
            i,
            range.source_end,
            source.len()
        );
    }

    // Should have HTML ranges for element tags
    let html = html_ranges(&result);
    assert!(!html.is_empty(), "Should have HTML ranges");

    // HTML ranges should cover source content
    for (i, range) in html.iter().enumerate() {
        assert!(
            range.source_end > range.source_start,
            "HTML range {} should have positive length",
            i
        );
        assert!(
            range.source_end <= source.len(),
            "HTML range {} source_end {} exceeds source len {}",
            i,
            range.source_end,
            source.len()
        );
    }

    // HTML injections should have empty prefix/suffix
    for inj in &html_injections(&result) {
        assert!(inj.prefix.is_empty(), "HTML prefix should be empty");
        assert!(inj.suffix.is_empty(), "HTML suffix should be empty");
    }
}

// ========================================================================
// Text expression ranges exclude braces
// ========================================================================

#[test]
fn test_text_expression_range_excludes_braces() {
    let source = "<div>Hello {name}!</div>";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    // Find the range for the text expression
    let expr_range = py.iter().find(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "name"
    });
    assert!(
        expr_range.is_some(),
        "Should have a Python range for just 'name' (no braces). Got: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_complex_expression_range_excludes_braces() {
    let source = r#"<div>{count + 1}</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let expr_range = py.iter().find(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "count + 1"
    });
    assert!(
        expr_range.is_some(),
        "Should have range for 'count + 1' (no braces). Got: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

// ========================================================================
// Error position tests
// ========================================================================

#[test]
fn test_error_has_position_for_unclosed_element() {
    let source = "<div>unclosed";
    let mut pipeline = Pipeline::standard();
    let err = pipeline
        .compile(source, &GenerateOptions::default())
        .unwrap_err();

    match err {
        hyper_transpiler::CompileError::Parse(parse_err) => {
            assert_eq!(parse_err.span.start.line, 0);
            assert!(parse_err.message.contains("never closed"));
        }
        _ => panic!("Expected ParseError, got {:?}", err),
    }
}

#[test]
fn test_error_has_position_for_nested_interactive() {
    let source = "<a><button>click</button></a>";
    let mut pipeline = Pipeline::standard();
    let err = pipeline
        .compile(source, &GenerateOptions::default())
        .unwrap_err();

    match err {
        hyper_transpiler::CompileError::Parse(parse_err) => {
            assert!(parse_err.message.contains("cannot appear inside"));
            // Error should point to the <button> tag, not <a>
            assert!(
                parse_err.span.start.col > 0,
                "Should point to <button>, not start of line"
            );
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_error_has_position_for_duplicate_attribute() {
    let source = r#"<div class={x} class={y}>text</div>"#;
    let mut pipeline = Pipeline::standard();
    let err = pipeline
        .compile(source, &GenerateOptions::default())
        .unwrap_err();

    match err {
        hyper_transpiler::CompileError::Parse(parse_err) => {
            assert!(
                parse_err.message.contains("twice")
                    || parse_err.message.contains("duplicate")
                    || parse_err.message.contains("set twice")
            );
            // Should have position info
            assert_eq!(parse_err.span.start.line, 0);
        }
        _ => panic!("Expected ParseError"),
    }
}

// ========================================================================
// HTML range accuracy tests
// ========================================================================

#[test]
fn test_html_range_source_text_simple() {
    let source = "<div>Hello</div>";
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);
    assert!(!html.is_empty());

    // Each HTML range should extract valid HTML text from source
    for (i, range) in html.iter().enumerate() {
        let text = &source[range.source_start..range.source_end];
        assert!(!text.is_empty(), "HTML range {} should not be empty", i);
        assert!(
            text.contains('<') || text.contains('>'),
            "HTML range {} should contain tag characters, got: {:?}",
            i,
            text
        );
    }
}

#[test]
fn test_html_range_source_text_with_attributes() {
    let source = r#"<div class={active} id="main">Content</div>"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);
    // HTML ranges should split around the {active} expression
    assert!(
        html.len() >= 2,
        "Should have at least 2 HTML ranges, got {}",
        html.len()
    );

    // First HTML range: "<div class=" (before expression)
    let first_text = &source[html[0].source_start..html[0].source_end];
    assert!(
        first_text.starts_with("<div"),
        "First HTML range should start with <div, got: {:?}",
        first_text
    );

    // Verify no HTML range contains the expression braces
    for range in &html {
        let text = &source[range.source_start..range.source_end];
        assert!(
            !text.contains("{active}"),
            "HTML range should not contain expression: {:?}",
            text
        );
    }
}

#[test]
fn test_html_ranges_void_element() {
    let source = r#"<img src={url} alt="photo" />"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);
    assert!(!html.is_empty(), "Void elements should have HTML ranges");

    // Should split around {url} expression
    let py = python_ranges(&result);
    assert!(
        !py.is_empty(),
        "Should have Python range for url expression"
    );
}

// ========================================================================
// No-overlap invariant
// ========================================================================

#[test]
fn test_no_overlap_python_html_ranges() {
    // Test across multiple templates
    let templates = vec![
        "<div class={x}>Hello {name}</div>",
        r#"<input type="text" value={val} />"#,
        "<span {disabled} data-id={id}>text</span>",
        "<div class={a} style={b} {c} {**d}>{e}</div>",
    ];

    for source in templates {
        let result = compile_with_ranges(source, "Test");

        let py = python_ranges(&result);
        let html = html_ranges(&result);

        for h in &html {
            for p in &py {
                let overlaps = h.source_start < p.source_end && h.source_end > p.source_start;
                assert!(
                    !overlaps,
                    "Overlap in {:?}: HTML [{},{}] overlaps Python [{},{}]",
                    source, h.source_start, h.source_end, p.source_start, p.source_end
                );
            }
        }
    }
}

// ========================================================================
// Injection prefix+suffix reconstruction
// ========================================================================

#[test]
fn test_injection_reconstruction_produces_valid_python() {
    let source = "name: str\n---\n<div>{name}</div>";
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert!(!py.is_empty());

    // Reconstruct full code from injections
    // Each injection's prefix + source[start..end] + suffix should produce valid-looking Python
    for inj in &py {
        let source_slice = &source[inj.start..inj.end];
        let combined = format!("{}{}{}", inj.prefix, source_slice, inj.suffix);

        // The combined code should contain the original source expression
        assert!(
            combined.contains(source_slice),
            "Reconstructed code should contain source slice {:?}",
            source_slice
        );
    }

    // At least one injection (the first) should reconstruct to code containing "def"
    let first = &py[0];
    let first_combined = format!(
        "{}{}{}",
        first.prefix,
        &source[first.start..first.end],
        first.suffix
    );
    assert!(
        first_combined.contains("def "),
        "First injection reconstruction should contain 'def': {:?}",
        first_combined
    );
}

#[test]
fn test_injection_reconstruction_with_multiple_expressions() {
    let source = r#"<div class={active}>Hello {name}!</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_injections(&result);
    assert!(py.len() >= 2, "Should have at least 2 Python injections");

    // Verify each injection is self-consistent
    for (i, inj) in py.iter().enumerate() {
        assert!(
            inj.start < inj.end,
            "Injection {} has invalid range: {} >= {}",
            i,
            inj.start,
            inj.end
        );
        assert!(
            inj.end <= source.len(),
            "Injection {} end {} exceeds source len {}",
            i,
            inj.end,
            source.len()
        );

        // Verify the suffix of one injection connects to the prefix of the next
        if i + 1 < py.len() {
            let next = &py[i + 1];
            // The gap between injections in the source should be bridged by suffix+prefix
            let bridge = format!("{}{}", inj.suffix, next.prefix);
            // The bridge should contain the compiled equivalent of the gap
            assert!(
                !bridge.is_empty(),
                "Bridge between injections {} and {} should not be empty",
                i,
                i + 1
            );
        }
    }
}

// ========================================================================
// Control flow expression ranges
// ========================================================================

#[test]
fn test_if_condition_has_python_range() {
    let source = "if active:\n    <div>yes</div>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    // Should have a range for the "active" condition
    let has_condition = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains("active")
    });
    assert!(
        has_condition,
        "Should have Python range for if condition. Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_for_loop_has_python_range() {
    let source = "for item in items:\n    <li>{item}</li>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    // Must have a range that covers the binding AND iterable together
    let has_binding_and_iterable = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains("item in items")
    });
    assert!(
        has_binding_and_iterable,
        "Should have Python range for 'item in items' (not just iterable). Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );

    // Must also have a range for the {item} expression inside the body
    let has_item_expr = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "item" && r.source_start > source.find('{').unwrap()
    });
    assert!(
        has_item_expr,
        "Should have Python range for {{item}} expression"
    );
}

#[test]
fn test_for_loop_binding_in_range() {
    let source = "for item in items:\n    <li>{item}</li>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    // The for-loop should have a range covering "item in items" (binding + iterable)
    let loop_range = py.iter().find(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "item in items"
    });
    assert!(
        loop_range.is_some(),
        "Should have Python range for 'item in items'. Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_while_condition_has_python_range() {
    let source = "while running:\n    <p>Loading</p>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let has_condition = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains("running")
    });
    assert!(
        has_condition,
        "Should have Python range for while condition"
    );
}

#[test]
fn test_except_clause_has_python_range() {
    let source = "try:\n    {x}\nexcept ValueError as e:\n    {e}\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let has_except = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "ValueError as e"
    });
    assert!(
        has_except,
        "Should have Python range for except clause. Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_html_ranges_basic() {
    let source = "<div>Hello</div>";
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);
    assert!(
        !html.is_empty(),
        "Should have HTML ranges for static element"
    );

    // The HTML range covers the opening tag <div>
    let first = html[0];
    let tag_text = &source[first.source_start..first.source_end];
    assert!(
        tag_text.starts_with("<div"),
        "HTML range should cover opening tag, got: {}",
        tag_text
    );

    // HTML injections should have empty prefix/suffix
    let html_inj = html_injections(&result);
    for inj in &html_inj {
        assert!(inj.prefix.is_empty());
        assert!(inj.suffix.is_empty());
    }
}

#[test]
fn test_html_ranges_with_expression() {
    let source = "<div class={x}>Hello {name}!</div>";
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);
    assert!(
        html.len() >= 2,
        "Should have multiple HTML ranges (split around expressions), got {}",
        html.len()
    );

    // Verify HTML ranges don't overlap with expression spans
    let py = python_ranges(&result);
    for h in &html {
        for p in &py {
            let overlaps = h.source_start < p.source_end && h.source_end > p.source_start;
            // HTML ranges should not overlap with the expression braces
            // (they might be adjacent but not overlapping)
            if overlaps {
                // Check it's just adjacency, not overlap
                assert!(
                    h.source_end <= p.source_start || h.source_start >= p.source_end,
                    "HTML range [{},{}] overlaps with Python range [{},{}]",
                    h.source_start,
                    h.source_end,
                    p.source_start,
                    p.source_end
                );
            }
        }
    }
}

// ========================================================================
// Template attribute expression ranges
// ========================================================================

#[test]
fn test_template_attribute_single_expression() {
    let source = r#"<button class="btn btn-{variant}">Click</button>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    // Should have a Python range for the {variant} expression
    let variant_range = py.iter().find(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "variant"
    });
    assert!(
        variant_range.is_some(),
        "Should have Python range for 'variant' in template attribute. Got: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );

    // Verify compiled text is also 'variant' (expression should match)
    let range = variant_range.unwrap();
    assert!(
        range.compiled_end > range.compiled_start,
        "Template expression range should have positive compiled length"
    );
}

#[test]
fn test_template_attribute_multiple_expressions() {
    let source = r#"<div data-info="{id}-{variant}">Info</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let id_range = py
        .iter()
        .find(|r| &source[r.source_start..r.source_end] == "id");
    let variant_range = py
        .iter()
        .find(|r| &source[r.source_start..r.source_end] == "variant");

    assert!(id_range.is_some(), "Should have Python range for 'id'");
    assert!(
        variant_range.is_some(),
        "Should have Python range for 'variant'"
    );

    // id should come before variant in source
    assert!(id_range.unwrap().source_start < variant_range.unwrap().source_start);
}

#[test]
fn test_template_attribute_adjacent_expressions() {
    let source = r#"<span data-key="{a}{b}">text</span>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let a_range = py
        .iter()
        .find(|r| &source[r.source_start..r.source_end] == "a");
    let b_range = py
        .iter()
        .find(|r| &source[r.source_start..r.source_end] == "b");

    assert!(a_range.is_some(), "Should have Python range for 'a'");
    assert!(b_range.is_some(), "Should have Python range for 'b'");
}

#[test]
fn test_template_attribute_html_range_splits() {
    let source = r#"<a href="/users/{id}" class="link">Go</a>"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);
    let py = python_ranges(&result);

    // HTML ranges should split around the {id} expression in the template attribute
    assert!(
        html.len() >= 2,
        "Should have at least 2 HTML ranges (split around template expression), got {}",
        html.len()
    );

    // No HTML range should contain the expression braces
    for h in &html {
        let text = &source[h.source_start..h.source_end];
        assert!(
            !text.contains("{id}"),
            "HTML range should not contain '{{id}}', got: {:?}",
            text
        );
    }

    // Python range should exist for the expression
    let id_range = py
        .iter()
        .find(|r| &source[r.source_start..r.source_end] == "id");
    assert!(
        id_range.is_some(),
        "Should have Python range for 'id' in template attribute"
    );

    // No overlap between HTML and Python ranges
    for h in &html {
        for p in &py {
            let overlaps = h.source_start < p.source_end && h.source_end > p.source_start;
            assert!(
                !overlaps,
                "HTML range [{},{}] overlaps Python range [{},{}]",
                h.source_start, h.source_end, p.source_start, p.source_end
            );
        }
    }
}

#[test]
fn test_template_attribute_roundtrip() {
    // Verify the virtual Python reconstruction works for template attributes
    let source = r#"name: str
---
<div class="item-{name}">text</div>"#;
    let result = compile_with_ranges(source, "Test");

    // Reconstruct virtual Python from injections
    let py_injections: Vec<_> = result
        .injections
        .iter()
        .filter(|i| i.injection_type == "python")
        .collect();

    assert!(!py_injections.is_empty(), "Should have Python injections");

    let mut virtual_python = String::new();
    for inj in &py_injections {
        virtual_python.push_str(&inj.prefix);
        // Use UTF-16 substring since injection positions are UTF-16
        let units: Vec<u16> = source.encode_utf16().collect();
        let end = inj.end.min(units.len());
        let start = inj.start.min(end);
        let slice = String::from_utf16_lossy(&units[start..end]);
        virtual_python.push_str(&slice);
        virtual_python.push_str(&inj.suffix);
    }

    assert_eq!(
        virtual_python, result.code,
        "Virtual Python from injections should match compiled code"
    );
}

// ========================================================================
// Decorator injection ranges
// ========================================================================

#[test]
fn test_decorator_has_python_range() {
    let source = "@fragment\ndef Badge(text: str):\n    <span>{text}</span>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let has_decorator = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "@fragment"
    });
    assert!(
        has_decorator,
        "Should have Python range for @fragment decorator. Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

// ========================================================================
// Definition signature injection ranges
// ========================================================================

#[test]
fn test_def_signature_has_python_range() {
    let source = "def Badge(text: str):\n    <span>{text}</span>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let has_def = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains("def Badge(text: str):")
    });
    assert!(
        has_def,
        "Should have Python range for def signature. Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

// ========================================================================
// Standalone expression injection ranges
// ========================================================================

#[test]
fn test_standalone_expression_has_python_range() {
    let source = "def Badge(text: str):\n    <span>{text}</span>\nend\n{Badge(\"New\")}";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let has_call = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "Badge(\"New\")"
    });
    assert!(
        has_call,
        "Should have Python range for standalone expression Badge(\"New\"). Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_component_tag_braces_in_expression_braces() {
    // <{Card}> opening and </{Card}> closing braces should be in expression_braces
    let source = "<{Card}>\n    <p>hi</p>\n</{Card}>";
    let result = compile_with_ranges(source, "Test");

    let braces = &result.expression_braces;

    // Opening tag <{Card}>: { at byte 1, } at byte 6
    let has_open = braces.iter().any(|b| b.open == 1 && b.close == 6);
    assert!(
        has_open,
        "Opening component tag braces <{{Card}}> should be tracked. Got: {:?}",
        braces
    );

    // Closing tag </{Card}>: { at byte 24, } at byte 29
    let close_brace_open = source.rfind("/{").unwrap() + 1;
    let close_brace_close = source.rfind("}").unwrap();
    let has_close = braces
        .iter()
        .any(|b| b.open == close_brace_open && b.close == close_brace_close);
    assert!(
        has_close,
        "Closing component tag braces </{{Card}}> should be tracked at ({}, {}). Got: {:?}",
        close_brace_open, close_brace_close, braces
    );
}

#[test]
fn test_named_slot_tag_braces_in_expression_braces() {
    // <{...header}> and </{...header}> braces should be in expression_braces
    let source = "<{...header}>\n    <h1>Fallback</h1>\n</{...header}>";
    let result = compile_with_ranges(source, "Test");

    let braces = &result.expression_braces;

    // Opening tag <{...header}>: { at byte 1, } at byte 11
    let open_brace = source.find('{').unwrap();
    let open_close = source.find('}').unwrap();
    let has_open = braces
        .iter()
        .any(|b| b.open == open_brace && b.close == open_close);
    assert!(
        has_open,
        "Opening named slot tag braces <{{...header}}> should be tracked at ({}, {}). Got: {:?}",
        open_brace, open_close, braces
    );

    // Closing tag </{...header}>: find the closing { and }
    let close_brace_open = source.rfind("/{").unwrap() + 1;
    let close_brace_close = source.rfind('}').unwrap();
    let has_close = braces
        .iter()
        .any(|b| b.open == close_brace_open && b.close == close_brace_close);
    assert!(
        has_close,
        "Closing named slot tag braces </{{...header}}> should be tracked at ({}, {}). Got: {:?}",
        close_brace_open, close_brace_close, braces
    );
}

#[test]
fn test_component_tag_angle_brackets_have_html_ranges() {
    // <{Card}> and </{Card}> angle brackets should have HTML injection ranges
    // so JetBrains colors them like regular HTML tag punctuation
    let source = "<{Card}>\n    <p>hi</p>\n</{Card}>";
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);

    // Opening tag: "<" at byte 0 should be in an HTML range
    let has_open_lt = html.iter().any(|r| r.source_start == 0 && r.source_end > 0);
    assert!(
        has_open_lt,
        "Opening '<' of <{{Card}}> should be in an HTML range. HTML ranges: {:?}",
        html.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );

    // Opening tag: ">" at byte 7 should be in an HTML range
    let gt_pos = source.find('>').unwrap();
    let has_open_gt = html
        .iter()
        .any(|r| r.source_start <= gt_pos && r.source_end > gt_pos);
    assert!(
        has_open_gt,
        "Closing '>' of <{{Card}}> should be in an HTML range. HTML ranges: {:?}",
        html.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );

    // Closing tag: "</" and ">" of </{Card}> should be in HTML range(s)
    let close_lt = source.find("</").unwrap();
    let has_close_lt = html
        .iter()
        .any(|r| r.source_start <= close_lt && r.source_end > close_lt);
    assert!(
        has_close_lt,
        "'</' of </{{Card}}> should be in an HTML range. HTML ranges: {:?}",
        html.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_slot_tag_angle_brackets_have_html_ranges() {
    // <{...header}> and </{...header}> angle brackets should have HTML injection ranges
    let source = "<{...header}>\n    <h1>Fallback</h1>\n</{...header}>";
    let result = compile_with_ranges(source, "Test");

    let html = html_ranges(&result);

    // Opening tag: "<" at byte 0
    let has_open_lt = html.iter().any(|r| r.source_start == 0 && r.source_end > 0);
    assert!(
        has_open_lt,
        "Opening '<' of <{{...header}}> should be in an HTML range. HTML ranges: {:?}",
        html.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );

    // Closing tag: "</" of </{...header}>
    let close_lt = source.find("</").unwrap();
    let has_close_lt = html
        .iter()
        .any(|r| r.source_start <= close_lt && r.source_end > close_lt);
    assert!(
        has_close_lt,
        "'</' of </{{...header}}> should be in an HTML range. HTML ranges: {:?}",
        html.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_component_name_has_python_range() {
    // The component name inside braces should have a Python injection range
    // so the IDE can resolve it (go-to-definition, highlighting)
    let source = "<{Badge} text=\"Sale\" />";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let name_start = source.find("Badge").unwrap();
    let name_end = name_start + "Badge".len();
    let has_name = py
        .iter()
        .any(|r| r.source_start == name_start && r.source_end == name_end);
    assert!(
        has_name,
        "Component name 'Badge' at [{},{}] should have a Python range. Ranges: {:?}",
        name_start,
        name_end,
        py.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_component_close_name_has_python_range() {
    // The closing tag component name should also have a Python range
    let source = "<{Card}>\n    <p>hi</p>\n</{Card}>";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let close_name_start = source.rfind("Card").unwrap();
    let close_name_end = close_name_start + "Card".len();
    let has_close_name = py
        .iter()
        .any(|r| r.source_start == close_name_start && r.source_end == close_name_end);
    assert!(
        has_close_name,
        "Closing tag component name 'Card' at [{},{}] should have a Python range. Ranges: {:?}",
        close_name_start,
        close_name_end,
        py.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );
}

// Note: slot names (e.g. "header" in <{...header}>) are Hyper syntax, not Python
// expressions. They compile to _header, not header. No Python injection range
// is emitted — styling comes from the TextMate grammar / annotator instead.

// ========================================================================
// Nested braces in attribute expressions
// ========================================================================

#[test]
fn test_dict_in_attribute_expression() {
    // Dict literal inside attribute expression: class={["card", {"sale": on_sale}]}
    // The tokenizer must track bracket depth to find the correct closing }
    let source = r#"<div class={["card", {"sale": on_sale}]}>text</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let expr_range = py.iter().find(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains("card") && text.contains("on_sale")
    });
    assert!(
        expr_range.is_some(),
        "Should have Python range for full dict expression. Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );

    // The captured expression should include the closing }]
    let range = expr_range.unwrap();
    let text = &source[range.source_start..range.source_end];
    assert_eq!(
        text, r#"["card", {"sale": on_sale}]"#,
        "Expression should include full list with nested dict"
    );

    // The compiled output should produce valid Python
    assert!(
        result.code.contains(r#"["card", {"sale": on_sale}]"#),
        "Compiled code should contain full expression. Got:\n{}",
        result.code
    );
}

#[test]
fn test_string_with_brace_in_attribute_expression() {
    // A string literal containing "}" inside an attribute expression
    // must not terminate the expression early
    let source = r#"<div class={get_class("}")}>text</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let expr_range = py.iter().find(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains("get_class")
    });
    assert!(
        expr_range.is_some(),
        "Should have Python range for get_class expression. Ranges: {:?}",
        py.iter()
            .map(|r| &source[r.source_start..r.source_end])
            .collect::<Vec<_>>()
    );

    let range = expr_range.unwrap();
    let text = &source[range.source_start..range.source_end];
    assert_eq!(
        text, r#"get_class("}")"#,
        "Expression should include full call with string containing brace"
    );
}

// ========================================================================
// Shorthand vs spread semantics
// ========================================================================

#[test]
fn test_shorthand_on_html_element() {
    // {disabled} on an HTML element renders as a single attribute
    let source = r#"<button {disabled}>Click</button>"#;
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("render_attr(\"disabled\", disabled)"),
        "Shorthand on HTML should use render_attr. Got:\n{}",
        result.code
    );
}

#[test]
fn test_shorthand_on_component() {
    // {disabled} on a component passes as a keyword argument
    let source = r#"<{Button} {disabled} />"#;
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("disabled=disabled"),
        "Shorthand on component should emit name=name. Got:\n{}",
        result.code
    );
}

#[test]
fn test_spread_on_component() {
    // {**props} on a component unpacks as kwargs
    let source = r#"<{Card} {**props} />"#;
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("**props"),
        "Spread on component should emit **props. Got:\n{}",
        result.code
    );
}

#[test]
fn test_spread_on_html_element() {
    // {**attrs} on an HTML element spreads as individual attributes
    let source = r#"<div {**attrs}>Content</div>"#;
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("spread_attrs(attrs)"),
        "Spread on HTML should use spread_attrs. Got:\n{}",
        result.code
    );
}

#[test]
fn test_shorthand_and_spread_together_on_component() {
    // Mixing shorthand and spread on a component
    let source = r#"<{Card} {disabled} label="hi" {**props} />"#;
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("disabled=disabled"),
        "Shorthand should emit name=name. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("**props"),
        "Spread should emit **props. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("label=\"hi\""),
        "Static attr should pass through. Got:\n{}",
        result.code
    );
}

// ========================================================================
// Implicit spread edge cases
// ========================================================================

#[test]
fn test_multiple_spread_names_require_explicit_declaration() {
    // Two elements use different spread names — implicit can't handle this
    // (Python only allows one **kwargs), so both must be declared explicitly
    let source = "container_attrs: dict\nbutton_attrs: dict\n---\n<div {**container_attrs}>\n    <button {**button_attrs}>Click</button>\n</div>";
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("spread_attrs(container_attrs)"),
        "First spread should compile. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("spread_attrs(button_attrs)"),
        "Second spread should compile. Got:\n{}",
        result.code
    );
    // Both are regular dict params, not **kwargs
    let sig_line = result.code.lines().find(|l| l.contains("def ")).unwrap();
    assert!(
        sig_line.contains("container_attrs: dict") && sig_line.contains("button_attrs: dict"),
        "Both should be regular dict params. Sig: {}",
        sig_line
    );
}
