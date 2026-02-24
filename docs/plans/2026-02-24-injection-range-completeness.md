# Injection Range Completeness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix two gaps in injection ranges (for-loop bindings and except clauses) so IDE navigation works correctly, and add semantic validation to prevent stale/broken ranges from being accepted.

**Architecture:** Add `binding_span` to `ForNode` in the parser, expand the for-loop injection range in the generator to cover `item in items` instead of just `items`, add the missing `add_range()` call for except clauses, and add source-text extraction validation to both the test suite and the `accept_expected` binary.

**Tech Stack:** Rust (parser, generator, tests), custom expected-output test harness

**Context:** See `docs/plans/2026-02-24-injection-range-completeness-design.md` for the full design rationale.

**Important:** This plan builds on recent changes in the current working tree (expression brace highlighting, control flow keyword highlighting in the plugin). Make sure you're on the same branch with those changes present.

---

### Task 1: Add `binding_span` to `ForNode`

**Files:**
- Modify: `rust/transpiler/src/ast.rs:128-135`
- Modify: `rust/transpiler/src/parser/tree_builder.rs:449-495`

**Step 1: Add the field to `ForNode`**

In `ast.rs`, add `binding_span` after `binding`:

```rust
pub struct ForNode {
    pub binding: String,
    pub binding_span: Span,       // NEW
    pub iterable: String,
    pub iterable_span: Span,
    pub body: Vec<Node>,
    pub is_async: bool,
    pub span: Span,
}
```

**Step 2: Compute `binding_span` in the parser**

In `tree_builder.rs` `parse_for`, after line 467 (`let iterable = ...`), add:

```rust
let binding_span = Span {
    start: rest_span.start,
    end: Position {
        line: rest_span.start.line,
        col: rest_span.start.col + parts[0].trim_start().len() + (parts[0].len() - parts[0].trim_start().len()),
        byte: rest_span.start.byte + parts[0].len(),
    },
};
```

And pass it into the `ForNode` constructor at line 487:

```rust
Ok(Some(Node::For(ForNode {
    binding,
    binding_span,   // NEW
    iterable,
    iterable_span,
    body,
    is_async,
    span: for_span,
})))
```

**Step 3: Run `cargo check` to find any missing field errors and fix them**

Any code that constructs `ForNode` in tests will need the new field. Fix all compilation errors.

**Step 4: Run tests**

Run: `cd rust && cargo test`
Expected: All tests pass (no behavior change yet).

**Step 5: Commit**

```
feat: add binding_span to ForNode for injection range tracking
```

---

### Task 2: Expand for-loop injection range to include binding

**Files:**
- Modify: `rust/transpiler/src/generate/python.rs` — `emit_for` (~line 1166)

**Step 1: Write a focused test in `injection_tests.rs`**

Add after the existing `test_for_loop_has_python_range`:

```rust
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
    assert!(loop_range.is_some(),
        "Should have Python range for 'item in items'. Ranges: {:?}",
        py.iter().map(|r| &source[r.source_start..r.source_end]).collect::<Vec<_>>());
}
```

**Step 2: Run test to verify it fails**

Run: `cd rust && cargo test test_for_loop_binding_in_range -- --nocapture`
Expected: FAIL — currently the range only covers `"items"`.

**Step 3: Modify `emit_for` to expand the range**

In `python.rs` `emit_for` (~line 1166), change the range to start from the binding:

Current code:
```rust
output.push(&for_node.binding);
output.push(" in ");
let iterable = for_node.iterable.trim_end_matches(':').trim();
let iter_start = output.position();
output.push(iterable);
let iter_end = output.position();
let source_end = for_node.iterable_span.start.byte + iterable.len();
output.add_range(Range {
    range_type: RangeType::Python,
    source_start: for_node.iterable_span.start.byte,
    source_end,
    compiled_start: iter_start,
    compiled_end: iter_end,
    needs_injection: true,
});
```

New code:
```rust
let binding_start = output.position();
output.push(&for_node.binding);
output.push(" in ");
let iterable = for_node.iterable.trim_end_matches(':').trim();
output.push(iterable);
let range_end = output.position();
let source_end = for_node.iterable_span.start.byte + iterable.len();
output.add_range(Range {
    range_type: RangeType::Python,
    source_start: for_node.binding_span.start.byte,
    source_end,
    compiled_start: binding_start,
    compiled_end: range_end,
    needs_injection: true,
});
```

**Step 4: Run tests**

Run: `cd rust && cargo test`
Expected: `test_for_loop_binding_in_range` passes. Some snapshot tests may need updating.

**Step 5: Review and accept snapshot changes**

Run: `cargo run --bin accept_expected` to regenerate expected files, then `git diff -- '*.expected.json'` to review. Verify the for-loop ranges now cover `"item in items"` or similar binding+iterable text.

**Step 6: Commit**

```
feat: expand for-loop injection range to include binding variable
```

---

### Task 3: Add injection range for except clauses

**Files:**
- Modify: `rust/transpiler/src/generate/python.rs` — `emit_try` (~line 1291)

**Step 1: Write a test**

```rust
#[test]
fn test_except_clause_has_python_range() {
    let source = "try:\n    {x}\nexcept ValueError as e:\n    {e}\nend";
    let result = compile_with_ranges(source, "Test");

    let py = python_ranges(&result);
    let has_except = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "ValueError as e"
    });
    assert!(has_except,
        "Should have Python range for except clause. Ranges: {:?}",
        py.iter().map(|r| &source[r.source_start..r.source_end]).collect::<Vec<_>>());
}
```

**Step 2: Run test to verify it fails**

Run: `cd rust && cargo test test_except_clause_has_python_range -- --nocapture`
Expected: FAIL.

**Step 3: Add `add_range()` in `emit_try`**

In `emit_try`, in the except clause handling (~line 1298-1310), change:

```rust
for except in &try_node.except_clauses {
    self.indent(output, indent);
    output.push("except");
    if let Some(exception) = &except.exception {
        output.push(" ");
        let exception = exception.trim_end_matches(':').trim();
        output.push(exception);
    }
    output.push(":");
    output.newline();
```

To:

```rust
for except in &try_node.except_clauses {
    self.indent(output, indent);
    output.push("except");
    if let Some(exception) = &except.exception {
        output.push(" ");
        let exception = exception.trim_end_matches(':').trim();
        let start = output.position();
        output.push(exception);
        let end = output.position();
        if let Some(ref exc_span) = except.exception_span {
            let source_end = exc_span.start.byte + exception.len();
            output.add_range(Range {
                range_type: RangeType::Python,
                source_start: exc_span.start.byte,
                source_end,
                compiled_start: start,
                compiled_end: end,
                needs_injection: true,
            });
        }
    }
    output.push(":");
    output.newline();
```

**Step 4: Run tests**

Run: `cd rust && cargo test`
Expected: All pass including the new test.

**Step 5: Review and accept snapshots**

Review `git diff`, verify except ranges look correct, then accept.

**Step 6: Commit**

```
feat: add injection range for except clause exception types
```

---

### Task 4: Fix the flawed for-loop test

**Files:**
- Modify: `rust/transpiler/tests/injection_tests.rs:552-565`

**Step 1: Replace the flawed test**

Replace `test_for_loop_has_python_range` at line 552:

```rust
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
    assert!(has_binding_and_iterable,
        "Should have Python range for 'item in items' (not just iterable). Ranges: {:?}",
        py.iter().map(|r| &source[r.source_start..r.source_end]).collect::<Vec<_>>());

    // Must also have a range for the {item} expression inside the body
    let has_item_expr = py.iter().any(|r| {
        let text = &source[r.source_start..r.source_end];
        text == "item" && r.source_start > source.find('{').unwrap()
    });
    assert!(has_item_expr, "Should have Python range for {{item}} expression");
}
```

**Step 2: Run tests**

Run: `cd rust && cargo test test_for_loop_has_python_range -- --nocapture`
Expected: PASS (since Task 2 already expanded the range).

**Step 3: Commit**

```
fix: replace flawed for-loop injection test with precise assertions
```

---

### Task 5: Add semantic validation to injection_validation_tests.rs

**Files:**
- Modify: `rust/transpiler/tests/injection_validation_tests.rs`

**Step 1: Add a new "semantic" trial**

After the existing `bounds::` trial registration (~line 266), add a new trial that validates source text extraction:

```rust
// semantic:: — verify source ranges extract to meaningful text (not mid-word)
trials.push(Trial::test(format!("semantic::{}", test_name), move || {
    let source = std::fs::read_to_string(&hyper_path_s)
        .map_err(|e| format!("Failed to read: {}", e))?;
    let options = GenerateOptions {
        function_name: Some(fn_name_s.clone()),
        include_ranges: true,
    };
    let mut pipeline = Pipeline::standard();
    let result = pipeline
        .compile(&source, &options)
        .map_err(|e| format!("Compile error: {}", e))?;

    // Check that every Python range extracts to text that starts and ends
    // at word boundaries (not mid-identifier)
    for range in &result.ranges {
        if range.source_start >= range.source_end {
            continue;
        }
        let text = source.get(range.source_start..range.source_end)
            .ok_or_else(|| format!(
                "Range [{}, {}] out of bounds for source len {}",
                range.source_start, range.source_end, source.len()
            ))?;

        // Check: range should not start mid-identifier
        if range.source_start > 0 {
            let prev_char = source.as_bytes()[range.source_start - 1] as char;
            let first_char = text.chars().next().unwrap_or(' ');
            if prev_char.is_alphanumeric() && first_char.is_alphanumeric() {
                return Err(format!(
                    "Range [{}, {}] starts mid-identifier: prev='{}', text={:?}",
                    range.source_start, range.source_end, prev_char, text
                ).into());
            }
        }

        // Check: range should not end mid-identifier
        if range.source_end < source.len() {
            let last_char = text.chars().last().unwrap_or(' ');
            let next_char = source.as_bytes()[range.source_end] as char;
            if last_char.is_alphanumeric() && next_char.is_alphanumeric() {
                return Err(format!(
                    "Range [{}, {}] ends mid-identifier: text={:?}, next='{}'",
                    range.source_start, range.source_end, text, next_char
                ).into());
            }
        }
    }
    Ok(())
}));
```

**Step 2: Run tests**

Run: `cd rust && cargo test semantic:: -- --nocapture`
Expected: All pass (our ranges should be correct after Tasks 2-3).

**Step 3: Commit**

```
test: add semantic validation for injection source ranges
```

---

### Task 6: Add validation to accept_expected binary

**Files:**
- Modify: `rust/transpiler/src/bin/accept_expected.rs`

**Step 1: Add a validation function**

Add after the imports:

```rust
/// Validate that all source ranges in the result extract to meaningful text.
/// Returns a list of warnings for any ranges that start/end mid-identifier.
fn validate_source_ranges(source: &str, result: &hyper_transpiler::GenerateResult) -> Vec<String> {
    let mut warnings = Vec::new();
    for range in &result.ranges {
        if range.source_start >= range.source_end || range.source_end > source.len() {
            continue;
        }
        let text = &source[range.source_start..range.source_end];

        if range.source_start > 0 {
            let prev = source.as_bytes()[range.source_start - 1] as char;
            let first = text.chars().next().unwrap_or(' ');
            if prev.is_alphanumeric() && first.is_alphanumeric() {
                warnings.push(format!(
                    "  WARNING: range [{}, {}] starts mid-identifier: ...{}|{}...",
                    range.source_start, range.source_end, prev, &text[..text.len().min(20)]
                ));
            }
        }

        if range.source_end < source.len() {
            let last = text.chars().last().unwrap_or(' ');
            let next = source.as_bytes()[range.source_end] as char;
            if last.is_alphanumeric() && next.is_alphanumeric() {
                warnings.push(format!(
                    "  WARNING: range [{}, {}] ends mid-identifier: ...{}|{}...",
                    range.source_start, range.source_end, &text[text.len().saturating_sub(20)..], next
                ));
            }
        }
    }
    warnings
}
```

**Step 2: Call it before writing expected.json**

In the success path, after re-compiling with `include_ranges: true`, add:

```rust
let warnings = validate_source_ranges(&source, &result);
if !warnings.is_empty() {
    eprintln!("  ⚠ Range warnings for {}:", file_path);
    for w in &warnings {
        eprintln!("{}", w);
    }
}
```

This prints warnings but still writes the file (so you see the issues without blocking the workflow). The semantic test in Task 5 will catch these as hard failures.

**Step 3: Run the accept tool to verify**

Run: `cd rust && cargo run --bin accept_expected`
Expected: No warnings for any file.

**Step 4: Commit**

```
feat: add source range validation to accept_expected
```

---

### Task 7: Regenerate all expected.json files and final verification

**Step 1: Regenerate all expected files**

Run: `cd rust && cargo run --bin accept_expected`

**Step 2: Review the diffs**

Run: `git diff -- '*.expected.json'`

For each file, verify:
- For-loop ranges now cover `"item in items"` (not just `"items"`)
- Except clause ranges cover exception types
- No ranges extract to mid-word text

**Step 3: Run the full test suite**

Run: `cd rust && cargo test`
Expected: ALL tests pass including new semantic validation.

**Step 4: Run clippy**

Run: `cd rust && cargo clippy`
Expected: 0 warnings.

**Step 5: Commit**

```
chore: regenerate expected.json files with corrected injection ranges
```

---

### Task 8: Update expression_braces collection for for-loop binding

**Files:**
- Modify: `rust/transpiler/src/generate/python.rs` — `collect_braces_node`

The `collect_braces_node` function currently doesn't recurse into for-loop bindings (the binding itself has no braces), but verify that the for-loop body is correctly traversed. No changes needed if it already recurses into `for_node.body`. Just verify.

**Step 1: Verify and run tests**

Run: `cd rust && cargo test`
Expected: All pass.

---
