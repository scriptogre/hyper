# Injection Range Completeness

## Problem

The transpiler's injection ranges have two gaps that break IDE navigation:

1. **For-loop bindings** — `for item in items:` only injects `items` (the iterable). The binding `item` ends up as prefix text in the virtual Python file, so the IDE can't Ctrl+Click between `{item}` and the `for` binding.

2. **Except clauses** — `except ValueError:` has no injection range at all. The exception type gets no Python intelligence.

Additionally, the test suite has structural problems that allowed these gaps to go unnoticed:

- `comprehensive.expected.json` was stale (byte positions off by ~3) and blindly accepted
- `test_for_loop_has_python_range` uses substring matching that passes accidentally (`"items"` contains `"item"`)
- No semantic validation that injection ranges extract to meaningful source text

## Design

### 1. Parser: Track binding span in ForNode

Add `binding_span: Span` to `ForNode`. In `parse_for`, record the span covering just the binding text (e.g., `item` or `i, item`).

### 2. Parser: Track exception span in ExceptClause

Add `exception_span: Option<Span>` to `ExceptClause`. In `parse_except` (or wherever except is parsed), record the span of the exception type text.

### 3. Generator: Expand for-loop injection range

Change `emit_for` to inject `item in items` instead of just `items`:
- Source range: `binding_span.start.byte` to `iterable_span.end` (after trimming colon)
- Compiled range: from where binding starts to where iterable ends in output

The prefix for the next injection will contain `for ` and the suffix context stays the same. The virtual Python file now has the binding as real source text, enabling Ctrl+Click navigation.

### 4. Generator: Add except clause injection range

In `emit_try`, when `except.exception` is present, add `output.add_range()` covering the exception type. Source range from `exception_span`.

### 5. Tests: Semantic validation

Add validation to the injection test suite:
- For every Python range, extract source text at `source[start:end]` and verify it matches expected content (not mid-word)
- Fix `test_for_loop_has_python_range` to check binding independently
- Regenerate `comprehensive.expected.json` after fixes

### 6. accept_expected safety

Add a validation pass to `accept_expected` that checks all source ranges in the JSON output extract to word-boundary-aligned text. Fail loudly if a range starts or ends mid-identifier.

## Files Modified

| File | Change |
|------|--------|
| `rust/transpiler/src/ast.rs` | Add `binding_span` to `ForNode`, `exception_span` to `ExceptClause` |
| `rust/transpiler/src/parser/tree_builder.rs` | Track binding and exception spans during parsing |
| `rust/transpiler/src/generate/python.rs` | Expand for-loop range, add except range |
| `rust/transpiler/tests/injection_tests.rs` | Fix flawed for-loop test, add semantic validation |
| `rust/transpiler/src/bin/accept_expected.rs` | Add source range validation before accepting |
| Various `.expected.json` files | Regenerated with correct ranges |
