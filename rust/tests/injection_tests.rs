mod common;

use common::{compile_with_ranges, html_segments, python_segments};
use hyper::CompileOptions;
use hyper::generate::Language;

#[test]
fn test_expression_segment() {
    let source = "<button aria={x}>y</button>";
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert_eq!(py_ranges.len(), 1, "Expected 1 Python segment");
    assert_eq!(py_ranges[0].language, Language::Python);
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

    let html = html_segments(&result);
    assert!(!html.is_empty(), "Should have HTML segments");
    for seg in &html {
        assert!(
            seg.html_prefix.is_none(),
            "Element HTML segment should not carry an html_prefix"
        );
    }
}

#[test]
fn test_parameter_segment() {
    let source = "x: str\n---\n<div>{x}</div>";
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert!(!py_ranges.is_empty(), "Expected at least 1 Python segment");

    for (i, range) in py_ranges.iter().enumerate() {
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
fn test_text_expression_segment() {
    let source = "<div>{name}</div>";
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
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
}

#[test]
fn test_class_attribute_segment() {
    let source = r#"<div class={active and "active"}>Content</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert_eq!(py_ranges.len(), 1);
    let source_text = &source[py_ranges[0].source_start..py_ranges[0].source_end];
    assert_eq!(source_text, r#"active and "active""#);
}

#[test]
fn test_style_attribute_segment() {
    let source = r#"<div style={{"color": color}}>Text</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert_eq!(py_ranges.len(), 1);
    let source_text = &source[py_ranges[0].source_start..py_ranges[0].source_end];
    assert!(
        source_text.starts_with("{"),
        "Should start with opening brace"
    );
    assert!(source_text.contains("color"), "Should contain 'color'");
}

#[test]
fn test_spread_attribute_segment() {
    let source = r#"<button aria={aria_attrs}>Close</button>"#;
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert_eq!(py_ranges.len(), 1);
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

    let py_ranges = python_segments(&result);
    // x in class, y in text, z in aria + params
    assert!(
        py_ranges.len() >= 3,
        "Expected at least 3 Python segments, got {}",
        py_ranges.len()
    );

    for (i, range) in py_ranges.iter().enumerate() {
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

    let html = html_segments(&result);
    assert!(!html.is_empty(), "Should have HTML ranges for element tags");
}

/// Tests that segments are correct when the template uses an explicit `---`
/// separator between parameters and body. Compare with `test_parameters_without_separator`.
#[test]
fn test_parameters_with_separator() {
    let source = r#"is_hidden: bool = False

---

aria_attrs = {"label": "Close dialog", "hidden": is_hidden, "live": "polite"}

<button aria={aria_attrs}>Close</button>"#;
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert!(!py_ranges.is_empty(), "Expected at least 1 Python segment");

    for range in &py_ranges {
        assert!(range.compiled_end > range.compiled_start);
        assert!(range.source_end > range.source_start);
    }
}

/// Tests that segments are correct when there is NO `---` separator.
/// The parser must infer where parameters end and body begins.
/// Compare with `test_parameters_with_separator`.
#[test]
fn test_parameters_without_separator() {
    let source = r#"is_hidden: bool = False

aria_attrs = {"label": "Close dialog", "hidden": is_hidden, "live": "polite"}

<button aria={aria_attrs}>Close</button>"#;
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert!(!py_ranges.is_empty(), "Expected at least 1 Python segment");

    for range in &py_ranges {
        assert!(range.compiled_end > range.compiled_start);
        assert!(range.source_end > range.source_start);
    }
}

#[test]
fn test_shorthand_attribute_segment() {
    let source = r#"<div {disabled}>Content</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py_ranges = python_segments(&result);
    assert_eq!(
        py_ranges.len(),
        1,
        "Expected 1 Python segment for shorthand"
    );
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

    let py_ranges = python_segments(&result);

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

    let html = html_segments(&result);
    assert!(!html.is_empty(), "Should have HTML ranges");

    // HTML ranges should cover source content; static element HTML carries no prefix.
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
        assert!(
            range.html_prefix.is_none(),
            "Element HTML segment should not carry an html_prefix"
        );
    }
}

// ========================================================================
// Text expression ranges exclude braces
// ========================================================================

#[test]
fn test_text_expression_range_excludes_braces() {
    let source = "<div>Hello {name}!</div>";
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
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

    let py = python_segments(&result);
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

    let err = hyper::compile(source, &CompileOptions::default()).unwrap_err();

    match err {
        hyper::CompileError::Parse(parse_err) => {
            assert_eq!(parse_err.range.start.line, 0);
            assert!(parse_err.message.contains("never closed"));
        }
        _ => panic!("Expected ParseError, got {:?}", err),
    }
}

#[test]
fn test_error_has_position_for_nested_interactive() {
    let source = "<a><button>click</button></a>";

    let err = hyper::compile(source, &CompileOptions::default()).unwrap_err();

    match err {
        hyper::CompileError::Parse(parse_err) => {
            assert!(parse_err.message.contains("cannot appear inside"));
            assert!(
                parse_err.range.start.col > 0,
                "Should point to <button>, not start of line"
            );
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_error_has_position_for_duplicate_attribute() {
    let source = r#"<div class={x} class={y}>text</div>"#;

    let err = hyper::compile(source, &CompileOptions::default()).unwrap_err();

    match err {
        hyper::CompileError::Parse(parse_err) => {
            assert!(
                parse_err.message.contains("twice")
                    || parse_err.message.contains("duplicate")
                    || parse_err.message.contains("set twice")
            );
            assert_eq!(parse_err.range.start.line, 0);
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

    let html = html_segments(&result);
    assert!(!html.is_empty());

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

    let html = html_segments(&result);
    assert!(
        html.len() >= 2,
        "Should have at least 2 HTML ranges, got {}",
        html.len()
    );

    let first_text = &source[html[0].source_start..html[0].source_end];
    assert!(
        first_text.starts_with("<div"),
        "First HTML range should start with <div, got: {:?}",
        first_text
    );

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

    let html = html_segments(&result);
    assert!(!html.is_empty(), "Void elements should have HTML ranges");

    let py = python_segments(&result);
    assert!(
        !py.is_empty(),
        "Should have Python range for url expression"
    );
}

// ========================================================================
// No-overlap invariant
// ========================================================================

#[test]
fn test_no_overlap_python_html_segments() {
    let templates = vec![
        "<div class={x}>Hello {name}</div>",
        r#"<input type="text" value={val} />"#,
        "<span {disabled} data-id={id}>text</span>",
        "<div class={a} style={b} {c} {**d}>{e}</div>",
    ];

    for source in templates {
        let result = compile_with_ranges(source, "Test");

        let py = python_segments(&result);
        let html = html_segments(&result);

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
// Control flow expression ranges
// ========================================================================

#[test]
fn test_if_condition_has_python_range() {
    let source = "if active:\n    <div>yes</div>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
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

    let py = python_segments(&result);
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

    let py = python_segments(&result);
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

    let py = python_segments(&result);
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

    let py = python_segments(&result);
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

    let html = html_segments(&result);
    assert!(
        !html.is_empty(),
        "Should have HTML ranges for static element"
    );

    let first = html[0];
    let tag_text = &source[first.source_start..first.source_end];
    assert!(
        tag_text.starts_with("<div"),
        "HTML range should cover opening tag, got: {}",
        tag_text
    );

    for seg in &html {
        assert!(
            seg.html_prefix.is_none(),
            "Element HTML segment should not carry an html_prefix"
        );
    }
}

#[test]
fn test_html_ranges_with_expression() {
    let source = "<div class={x}>Hello {name}!</div>";
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);
    assert!(
        html.len() >= 2,
        "Should have multiple HTML ranges (split around expressions), got {}",
        html.len()
    );

    let py = python_segments(&result);
    for h in &html {
        for p in &py {
            let overlaps = h.source_start < p.source_end && h.source_end > p.source_start;
            if overlaps {
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

    let py = python_segments(&result);
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

    let py = python_segments(&result);
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

    assert!(id_range.unwrap().source_start < variant_range.unwrap().source_start);
}

#[test]
fn test_template_attribute_adjacent_expressions() {
    let source = r#"<span data-key="{a}{b}">text</span>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
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

    let html = html_segments(&result);
    let py = python_segments(&result);

    assert!(
        html.len() >= 2,
        "Should have at least 2 HTML ranges (split around template expression), got {}",
        html.len()
    );

    for h in &html {
        let text = &source[h.source_start..h.source_end];
        assert!(
            !text.contains("{id}"),
            "HTML range should not contain '{{id}}', got: {:?}",
            text
        );
    }

    let id_range = py
        .iter()
        .find(|r| &source[r.source_start..r.source_end] == "id");
    assert!(
        id_range.is_some(),
        "Should have Python range for 'id' in template attribute"
    );

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

// ========================================================================
// Decorator injection ranges
// ========================================================================

#[test]
fn test_decorator_has_python_range() {
    let source = "@cache\ndef Badge(text: str):\n    <span>{text}</span>\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
    let has_decorator = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "@cache"
    });
    assert!(
        has_decorator,
        "Should have Python range for @cache decorator. Ranges: {:?}",
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

    let py = python_segments(&result);
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

    let py = python_segments(&result);
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
    let source = "<{Card}>\n    <p>hi</p>\n</{Card}>";
    let result = compile_with_ranges(source, "Test");

    let braces = &result.expression_braces;

    let has_open = braces.iter().any(|b| b.open == 1 && b.close == 6);
    assert!(
        has_open,
        "Opening component tag braces <{{Card}}> should be tracked. Got: {:?}",
        braces
    );

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
    let source = "<{...header}>\n    <h1>Fallback</h1>\n</{...header}>";
    let result = compile_with_ranges(source, "Test");

    let braces = &result.expression_braces;

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
fn test_component_named_fill_braces_survive_binding() {
    let source = r#"<{Card}>
    <button {...actions}>Save</button>
</{Card}>
<{Card}>
    <{...actions}>
        <button>Save</button>
    </{...actions}>
</{Card}>"#;
    let result = compile_with_ranges(source, "Test");
    let markers: Vec<_> = source.match_indices("{...actions}").collect();

    assert_eq!(markers.len(), 3);
    for (open, marker) in markers {
        let close = open + marker.len() - 1;
        assert!(
            result
                .expression_braces
                .iter()
                .any(|braces| braces.open == open && braces.close == close),
            "Named fill braces should be tracked at ({open}, {close}). Got: {:?}",
            result.expression_braces
        );
    }
}

#[test]
fn test_component_tag_no_lone_angle_bracket_html() {
    // Component tags emit no lone "<"/">"/"</" segments (unparseable HTML); the attr region carries an "<x" prefix.
    let source = "<{Card}>\n    <p>hi</p>\n</{Card}>";
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);

    let has_lone_lt = html
        .iter()
        .any(|r| r.source_start == 0 && r.source_end == 1);
    assert!(
        !has_lone_lt,
        "Should NOT have a lone '<' HTML range. HTML ranges: {:?}",
        html.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );

    let gt_range = html.iter().find(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains(">") && r.source_start > 0 && r.source_start < source.find('\n').unwrap()
    });
    assert!(
        gt_range.is_some(),
        "Should have HTML range for '>' with prefix. HTML ranges: {:?}",
        html.iter()
            .map(|r| (
                r.source_start,
                r.source_end,
                &source[r.source_start..r.source_end]
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        gt_range.unwrap().html_prefix.as_deref(),
        Some("<x"),
        "Component '>' HTML range should have '<x' prefix"
    );
}

#[test]
fn test_slot_tag_no_html_segments() {
    // Slot tags <{...header}> have no attributes, so they produce no
    // HTML ranges for the opening/closing tag (only for child content).
    let source = "<{...header}>\n    <h1>Fallback</h1>\n</{...header}>";
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);

    for r in &html {
        let text = &source[r.source_start..r.source_end];
        assert!(
            !text.contains("...header"),
            "Should not have HTML range containing slot name. Got: {:?}",
            text
        );
    }
}

#[test]
fn test_component_name_has_python_range() {
    let source = "<{Badge} text=\"Sale\" />";
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
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
    let source = "<{Card}>\n    <p>hi</p>\n</{Card}>";
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
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

// ========================================================================
// Nested braces in attribute expressions
// ========================================================================

#[test]
fn test_dict_in_attribute_expression() {
    let source = r#"<div class={["card", {"sale": on_sale}]}>text</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
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

    let range = expr_range.unwrap();
    let text = &source[range.source_start..range.source_end];
    assert_eq!(
        text, r#"["card", {"sale": on_sale}]"#,
        "Expression should include full list with nested dict"
    );

    assert!(
        result.code.contains(r#"["card", {"sale": on_sale}]"#),
        "Compiled code should contain full expression. Got:\n{}",
        result.code
    );
}

#[test]
fn test_string_with_brace_in_attribute_expression() {
    let source = r#"<div class={get_class("}")}>text</div>"#;
    let result = compile_with_ranges(source, "Test");

    let py = python_segments(&result);
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
    let source = r#"<{Card} {**props} />"#;
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("Card.stream(**props)"),
        "Spread on component should stream Card(**props). Got:\n{}",
        result.code
    );
}

#[test]
fn test_spread_on_html_element() {
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
// Implicit spread injection tests
// ========================================================================

#[test]
fn test_implicit_spread_all_blessed_names() {
    for name in &["props", "kwargs", "rest", "attrs", "attributes"] {
        let source = format!("<{{Card}} {{**{name}}} />");
        let result = compile_with_ranges(&source, "Test");

        let expected = format!("**{name},");
        assert!(
            result.code.contains(&expected),
            "Blessed name '{name}' should be auto-injected. Got:\n{}",
            result.code
        );
    }
}

#[test]
fn test_non_blessed_spread_no_injection() {
    let source = "my_dict: dict\n---\n<{Card} {**my_dict} />\n";
    let result = compile_with_ranges(source, "Test");

    assert!(
        !result.code.contains("**my_dict):\n"),
        "Non-blessed name should NOT be injected into signature. Got:\n{}",
        result.code
    );
    assert!(
        result.code.contains("Card.stream(**my_dict)"),
        "Should still stream Card(**my_dict) at the call site. Got:\n{}",
        result.code
    );
}

#[test]
fn test_explicit_param_prevents_injection() {
    let source = "props: dict\n---\n<{Card} {**props} />\n";
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("Card.stream(**props)"),
        "Should stream Card(**props) at the call site. Got:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("**props):"),
        "Should NOT inject **props when props is explicitly declared. Got:\n{}",
        result.code
    );
}

#[test]
fn test_explicit_starstar_param_prevents_injection() {
    let source = "**props\n---\n<{Card} {**props} />\n";
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("**props)"),
        "Should have **props in signature from explicit declaration. Got:\n{}",
        result.code
    );
}

#[test]
fn test_same_blessed_name_multiple_components() {
    let source = "<{Card} {**props} />\n<{Button} {**props} />\n";
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("**props,"),
        "Should inject **props once. Got:\n{}",
        result.code
    );
}

#[test]
fn test_multiple_different_blessed_names_error() {
    let source = "<{Card} {**props} />\n<{Button} {**attrs} />\n";

    let err = hyper::compile(source, &CompileOptions::default()).unwrap_err();

    match err {
        hyper::CompileError::Generate(msg) => {
            assert!(
                msg.contains("props") && msg.contains("attrs"),
                "Error should mention both spread names. Got: {msg}"
            );
        }
        _ => panic!("Expected Generate error, got: {:?}", err),
    }
}

#[test]
fn test_implicit_spread_with_regular_params() {
    let source =
        "title: str\ncount: int = 0\n---\n<{Card} title={title} count={count} {**props} />\n";
    let result = compile_with_ranges(source, "Test");

    assert!(
        result.code.contains("*,")
            && result.code.contains("title: str,")
            && result.code.contains("count: int = 0,")
            && result.code.contains("**props,"),
        "Should have regular params AND injected **props. Got:\n{}",
        result.code
    );
}

// ========================================================================
// Component tag HTML ranges must not overlap with Python attribute ranges
// ========================================================================

#[test]
fn test_component_html_ranges_no_overlap_with_expression_attrs() {
    let source = r#"<{Badge} text={format_name(name)} />"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);
    let py = python_segments(&result);

    for h in &html {
        for p in &py {
            let overlaps = h.source_start < p.source_end && h.source_end > p.source_start;
            assert!(
                !overlaps,
                "Component tag: HTML range [{},{}] ({:?}) overlaps Python range [{},{}] ({:?})",
                h.source_start,
                h.source_end,
                &source[h.source_start..h.source_end],
                p.source_start,
                p.source_end,
                &source[p.source_start..p.source_end]
            );
        }
    }
}

#[test]
fn test_component_html_ranges_no_overlap_with_shorthand_attrs() {
    let source = r#"<{Badge} {is_active} />"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);
    let py = python_segments(&result);

    for h in &html {
        for p in &py {
            let overlaps = h.source_start < p.source_end && h.source_end > p.source_start;
            assert!(
                !overlaps,
                "Component tag: HTML range [{},{}] ({:?}) overlaps Python range [{},{}] ({:?})",
                h.source_start,
                h.source_end,
                &source[h.source_start..h.source_end],
                p.source_start,
                p.source_end,
                &source[p.source_start..p.source_end]
            );
        }
    }
}

#[test]
fn test_component_with_mixed_attrs_html_splits() {
    let source = r#"<{Badge} text="Sale" badge_variant="danger" />"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);

    let has_text_attr = html.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text.contains("text=")
    });
    assert!(
        has_text_attr,
        "Should have HTML range covering 'text=' attribute. HTML ranges: {:?}",
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
fn test_component_attr_html_has_tag_prefix() {
    // Attribute fragments need a synthetic tag prefix to parse; the transpiler marks
    // the first attribute-region segment with html_prefix = Some("<x").
    let source = r#"<{Badge} text="Sale" badge_variant="danger" />"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);
    let prefixed: Vec<_> = html
        .iter()
        .filter(|s| s.html_prefix.as_deref() == Some("<x"))
        .collect();
    assert_eq!(
        prefixed.len(),
        1,
        "Expected 1 HTML segment with '<x' prefix. Got: {:?}",
        html.iter()
            .map(|s| (&source[s.source_start..s.source_end], &s.html_prefix))
            .collect::<Vec<_>>()
    );

    let text = &source[prefixed[0].source_start..prefixed[0].source_end];
    assert!(
        text.contains("text="),
        "Prefixed HTML segment should cover attributes, got: {:?}",
        text
    );
}

#[test]
fn test_component_no_attrs_has_tag_prefix() {
    // A bare component <{Badge} /> with no real attributes still gets the " />"
    // fragment marked with html_prefix = "<x" so it parses as <x />.
    let source = r#"<{Badge} />"#;
    let result = compile_with_ranges(source, "Test");

    let html = html_segments(&result);
    let prefixed: Vec<_> = html
        .iter()
        .filter(|s| s.html_prefix.as_deref() == Some("<x"))
        .collect();
    assert_eq!(
        prefixed.len(),
        1,
        "Expected 1 HTML segment with '<x' prefix. Got: {:?}",
        html.iter()
            .map(|s| (&source[s.source_start..s.source_end], &s.html_prefix))
            .collect::<Vec<_>>()
    );
}
