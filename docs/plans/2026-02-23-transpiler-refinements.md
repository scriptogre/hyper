# Transpiler Refinements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix correctness bugs in the marker system and decorator, improve codegen output, and clean up the API surface.

**Architecture:** Replace runtime marker processing with direct function calls in generated f-strings. Rework the `@html` decorator to use `functools.wraps` for type checker compatibility. Rename `_class` → `class_` per PEP 8.

**Tech Stack:** Rust (transpiler codegen), Python (runtime), `cargo test` + `pytest`

---

### Task 1: Rework the `@html` Decorator (sync)

**Files:**
- Modify: `python/hyper/decorators.py`
- Create: `python/tests/test_decorators.py`

**Step 1: Write failing tests for the new decorator behavior**

Create `python/tests/test_decorators.py`:

```python
import functools
import inspect
from hyper.decorators import html


def test_html_preserves_function_name():
    @html
    def MyComponent(*, title: str = ""):
        yield f"<h1>{title}</h1>"

    assert MyComponent.__name__ == "MyComponent"


def test_html_preserves_signature():
    @html
    def MyComponent(*, title: str = "", count: int = 0):
        yield f"<h1>{title}</h1>"

    sig = inspect.signature(MyComponent)
    assert "title" in sig.parameters
    assert "count" in sig.parameters


def test_html_result_str():
    @html
    def MyComponent(*, title: str = "World"):
        yield f"<h1>{title}</h1>"

    assert str(MyComponent(title="Hello")) == "<h1>Hello</h1>"


def test_html_result_iter():
    @html
    def MyComponent():
        yield "<p>one</p>"
        yield "<p>two</p>"

    chunks = list(MyComponent())
    assert chunks == ["<p>one</p>", "<p>two</p>"]


def test_html_result_yield_from():
    @html
    def Inner(*, text: str = ""):
        yield f"<span>{text}</span>"

    @html
    def Outer():
        yield "<div>"
        yield from Inner(text="hello")
        yield "</div>"

    assert str(Outer()) == "<div><span>hello</span></div>"


def test_html_rejects_positional_args():
    @html
    def MyComponent(*, title: str = ""):
        yield f"<h1>{title}</h1>"

    import pytest
    with pytest.raises(TypeError):
        MyComponent("oops")


def test_html_is_not_a_class():
    @html
    def MyComponent():
        yield "<p>hi</p>"

    assert not isinstance(MyComponent, type)
    assert callable(MyComponent)
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/chris/Projects/hyper && uv run pytest python/tests/test_decorators.py -v`

Expected: Several failures (signature not preserved, isinstance check fails, positional arg accepted silently).

**Step 3: Rewrite the sync decorator**

Replace `_make_sync_wrapper` and `SyncComponentWrapper` in `python/hyper/decorators.py` with:

```python
import functools
import inspect

__all__ = ["html"]


class HtmlResult:
    """Iterable, str()-able result from an @html component."""
    __slots__ = ("_fn", "_args", "_kwargs")

    def __init__(self, fn, args, kwargs):
        self._fn = fn
        self._args = args
        self._kwargs = kwargs

    def __iter__(self):
        return iter(self._fn(*self._args, **self._kwargs))

    def __str__(self):
        return "".join(self._fn(*self._args, **self._kwargs))

    def __repr__(self):
        return f"HtmlResult({self._fn.__name__})"


def html(fn):
    """Decorator that wraps a generator function for HTML template output.

    The wrapped function returns an HtmlResult that supports:
    - str(result) for buffered output
    - iter(result) / yield from result for streaming
    """
    if inspect.isasyncgenfunction(fn):
        return _make_async_wrapper(fn)

    @functools.wraps(fn)
    def wrapper(*args, **kwargs):
        return HtmlResult(fn, args, kwargs)
    return wrapper
```

Keep `_make_async_wrapper` for now — it will be updated in a later task if needed.

**Step 4: Run tests to verify they pass**

Run: `cd /Users/chris/Projects/hyper && uv run pytest python/tests/test_decorators.py -v`

Expected: All PASS.

**Step 5: Run the full transpiler test suite to check nothing breaks**

Run: `cd /Users/chris/Projects/hyper/rust && cargo test`

Expected: All pass (the generated .py files still use `@html` the same way — only the runtime behavior of the decorator changed).

**Step 6: Commit**

```bash
git add python/hyper/decorators.py python/tests/test_decorators.py
git commit -m "Rework @html decorator to preserve function metadata"
```

---

### Task 2: Replace ESCAPE Markers with `escape()` Calls

This is the largest change. Update the Rust codegen to emit `{escape(expr)}` instead of `‹ESCAPE:{expr}›`.

**Files:**
- Modify: `rust/transpiler/src/generate/python.rs` (lines ~224-240 — expression emission)
- Modify: All `rust/transpiler/tests/**/*.expected.py` files (via `cargo run --bin accept_expected`)
- Modify: All `rust/transpiler/tests/**/*.expected.json` files (via `cargo run --bin accept_expected`)

**Step 1: Update expression emission in the codegen**

In `rust/transpiler/src/generate/python.rs`, find the ESCAPE marker emission (~line 224). Replace:

```rust
// Old: ‹ESCAPE:{expr}›
output.push("‹ESCAPE:");
let start = output.position();
output.push("{");
output.push(&expr.expr);
output.push("}");
let end = output.position();
output.push("›");
```

With:

```rust
// New: {escape(expr)}
output.push("{escape(");
let start = output.position();
output.push(&expr.expr);
let end = output.position();
output.push(")}");
```

Note: the range tracking (`start`/`end`) now wraps just the expression, not `{expr}`. Adjust accordingly.

**Step 2: Update the generated import line**

In `python.rs`, find where `from hyper import html, replace_markers` is emitted. Change it to import `escape` instead of `replace_markers`. The import logic should be:

- Always: `from hyper import html`
- If any expressions: add `escape`
- If class attrs: add `render_class`
- If bool attrs: add `render_attr`
- etc.

Find the import emission code (look for `"from hyper import"` string in python.rs) and update accordingly.

**Step 3: Change `replace_markers(f"""...)` → `f"""...`**

The `yield replace_markers(f"""...""")` wrapper is no longer needed since markers are gone. Remove the `replace_markers(` prefix and `)` suffix from yield statements that contain expressions.

Find all places in python.rs where `replace_markers` is pushed to output and remove them. Yields with expressions should just be `yield f"""..."""`.

**Step 4: Run tests to see what changed**

Run: `cd /Users/chris/Projects/hyper/rust && cargo test 2>&1 | head -60`

Expected: Many output test failures (expected files still have old marker format).

**Step 5: Regenerate expected files and REVIEW the diffs**

Run:
```bash
cd /Users/chris/Projects/hyper/rust && cargo run --bin accept_expected
```

Then **carefully review** the diffs:
```bash
git diff -- '*.expected.py' | head -200
git diff -- '*.expected.json' | head -200
```

Verify:
- `replace_markers(f"""...‹ESCAPE:{x}›...""")` → `f"""...{escape(x)}..."""`
- `from hyper import html, replace_markers` → `from hyper import html, escape`
- Static-only yields (`yield """..."""`) remain unchanged (no `escape` wrapping)

**Step 6: Run tests again**

Run: `cd /Users/chris/Projects/hyper/rust && cargo test`

Expected: All PASS.

**Step 7: Commit**

```bash
git add rust/transpiler/src/generate/python.rs
git add rust/transpiler/tests/
git commit -m "Replace ESCAPE markers with direct escape() calls in codegen"
```

---

### Task 3: Replace Attribute Markers with Direct Function Calls

**Files:**
- Modify: `rust/transpiler/src/generate/python.rs` (lines ~341-466 — attribute marker emission)
- Modify: All `rust/transpiler/tests/**/*.expected.py` and `*.expected.json`

**Step 1: Replace CLASS marker emission**

In `python.rs` (~line 341), replace:

```rust
// Old: =‹CLASS:{expr}›
output.push("=‹CLASS:{");
...
output.push("}›");
```

With:

```rust
// New: ="{render_class(expr)}"
output.push("=\"{render_class(");
...
output.push(")}\"");
```

**Step 2: Replace BOOL marker emission**

In `python.rs` (~line 369), replace:

```rust
// Old: attrname=‹BOOL:{expr}›
output.push("=‹BOOL:{");
...
output.push("}›");
```

With:

```rust
// New: {render_attr("attrname", expr)}
// Note: this replaces the ENTIRE attribute (name + value), not just the value
// So the attribute name push before this needs to be removed/moved
output.push("{render_attr(\"");
output.push(name);
output.push("\", ");
...
output.push(")}");
```

Important: Boolean attributes need the attribute name moved inside `render_attr()`, so the `output.push(name); output.push("=")` before the BOOL marker must be removed.

**Step 3: Replace STYLE, DATA, ARIA, SPREAD marker emission**

Apply the same pattern for each remaining marker type:

- STYLE: `style=‹STYLE:{expr}›` → `style="{render_style(expr)}"`
- DATA: `data=‹DATA:{expr}›` → `{render_data(expr)}`
- ARIA: `aria=‹ARIA:{expr}›` → `{render_aria(expr)}`
- SPREAD: `‹SPREAD:{expr}›` → `{spread_attrs(expr)}`

For DATA, ARIA, and SPREAD: the attribute name is absorbed by the function (they expand to multiple attributes), so the name prefix must be removed from the output.

**Step 4: Update import emission**

The generated `from hyper import ...` line needs to include the helper functions actually used. Update the import logic to track which helpers are needed:

```rust
let mut needs_escape = false;
let mut needs_render_class = false;
let mut needs_render_attr = false;
let mut needs_render_style = false;
let mut needs_render_data = false;
let mut needs_render_aria = false;
let mut needs_spread_attrs = false;
```

Set these flags during codegen, then emit the appropriate import line.

**Step 5: Regenerate expected files and REVIEW**

```bash
cd /Users/chris/Projects/hyper/rust && cargo run --bin accept_expected
git diff -- '*.expected.py' | head -300
git diff -- '*.expected.json' | head -300
```

Verify each marker type was replaced correctly.

**Step 6: Run full test suite**

Run: `cd /Users/chris/Projects/hyper/rust && cargo test`

Expected: All PASS.

**Step 7: Commit**

```bash
git add rust/transpiler/src/generate/python.rs
git add rust/transpiler/tests/
git commit -m "Replace attribute markers with direct function calls in codegen"
```

---

### Task 4: Remove `replace_markers` from Python Runtime

**Files:**
- Modify: `python/hyper/helpers.py` (delete lines 22-31, 282-366)
- Modify: `python/hyper/__init__.py` (remove `replace_markers` from imports and `__all__`)

**Step 1: Delete marker code from helpers.py**

Remove:
- `_ATTR_MARKER_PATTERN` regex (line 26)
- `_ESCAPE_MARKER_PATTERN` regex (line 31)
- `import ast` (line 8 — only used by `replace_markers`)
- `replace_markers()` function (lines 282-366)
- `'replace_markers'` from `__all__` list

**Step 2: Update __init__.py**

Remove `replace_markers` from the import list and `__all__`.

**Step 3: Run Python tests**

Run: `cd /Users/chris/Projects/hyper && uv run pytest python/tests/ -v`

Expected: All PASS.

**Step 4: Run Rust tests (to verify generated code doesn't reference replace_markers)**

Run: `cd /Users/chris/Projects/hyper/rust && cargo test`

Expected: All PASS (no generated code should import `replace_markers` anymore after Task 2-3).

**Step 5: Commit**

```bash
git add python/hyper/helpers.py python/hyper/__init__.py
git commit -m "Remove replace_markers and marker regex patterns from runtime"
```

---

### Task 5: Rename `_class` to `class_` (PEP 8)

**Files:**
- Modify: `rust/transpiler/src/generate/python.rs` (keyword renaming logic)
- Modify: All affected `*.expected.py` test files

**Step 1: Find and update the keyword renaming logic**

In `python.rs`, search for `_class` or the logic that prefixes reserved keywords with `_`. Change it to append `_` (suffix) instead of prepend.

Look for code like:
```rust
format!("_{}", name)  // or similar
```

Change to:
```rust
format!("{}_", name)  // trailing underscore
```

This affects: parameter names, variable references, attribute shorthand expansion.

**Step 2: Regenerate expected files and review**

```bash
cd /Users/chris/Projects/hyper/rust && cargo run --bin accept_expected
git diff -- '*.expected.py'
```

Verify: `_class` → `class_`, `_type` → `type_` (if any).

**Step 3: Run tests**

Run: `cd /Users/chris/Projects/hyper/rust && cargo test`

Expected: All PASS.

**Step 4: Commit**

```bash
git add rust/transpiler/src/generate/python.rs rust/transpiler/tests/
git commit -m "Rename reserved keyword prefix from _class to class_ (PEP 8)"
```

---

### Task 6: Update Design Documentation

**Files:**
- Modify: `docs/design/templates.md`
- Modify: `docs/implementation/templates.md`
- Modify: `DECISIONS.md`

**Step 1: Remove HTML-in-expressions from design doc**

In `docs/design/templates.md`:
- Remove the "List Comprehensions" section (~lines 287-325) that shows `{[<li>{x}</li> for x in items]}`
- Remove the ternary-with-HTML examples (~lines 343-372) that show `{<span>text</span> if cond}`
- Remove the "Short-Circuit Evaluation" example with HTML (~lines 376-395) that shows `{show_warning and <p>...}`
- Keep the "When to Use What" table but remove rows referencing HTML-in-expressions
- Keep simple Python expression examples (`{"item" if count == 1 else "items"}`, `{title or "Untitled"}`)

**Step 2: Update implementation doc**

In `docs/implementation/templates.md`:
- Update the codegen examples to show `escape()` instead of `‹ESCAPE:...›`
- Update the attribute examples to show `render_class()`, `render_attr()` etc.
- Remove references to `replace_markers()`
- Update the "Security" section to show `escape()` instead of markers

**Step 3: Update DECISIONS.md**

Add a new decision entry:

```markdown
### 019: Direct Function Calls Replace Markers

| | |
|--|--|
| **Context** | Markers (`‹ESCAPE:{expr}›`) round-tripped Python values through string serialization. `safe()` was broken (f-string stringified the Safe object before escape_html saw it). `ast.literal_eval` failed for non-literal reprs. |
| **Decision** | Replace all markers with direct function calls in f-strings. `{escape(expr)}` for content, `{render_class(expr)}` for class, `{render_attr("name", expr)}` for booleans, etc. |
| **Trade-off** | Generated output diverges slightly more from source HTML structure, but IDE injection mapping still works (prefix/suffix mechanism unchanged). Eliminates `replace_markers()`, regex patterns, and `ast.literal_eval`. |
```

**Step 4: Commit**

```bash
git add docs/design/templates.md docs/implementation/templates.md DECISIONS.md
git commit -m "Update docs for direct function calls and remove HTML-in-expressions"
```

---

### Task 7: Verify Everything End-to-End

**Files:** None (verification only)

**Step 1: Run the full Rust test suite**

```bash
cd /Users/chris/Projects/hyper/rust && cargo test
```

Expected: All PASS.

**Step 2: Run Python tests**

```bash
cd /Users/chris/Projects/hyper && uv run pytest python/tests/ -v
```

Expected: All PASS.

**Step 3: Compile a real .hyper file and inspect output**

```bash
cd /Users/chris/Projects/hyper && just compile playground/
```

Inspect the generated .py files — verify no markers remain, imports are correct, `escape()` calls wrap expressions.

**Step 4: Spot-check that safe() works**

Create a quick test:

```bash
cd /Users/chris/Projects/hyper && uv run python -c "
from hyper import html, escape, safe

@html
def Test():
    trusted = safe('<b>bold</b>')
    yield f'<div>{escape(trusted)}</div>'

print(str(Test()))
# Should output: <div><b>bold</b></div>
# NOT: <div>&lt;b&gt;bold&lt;/b&gt;</div>
"
```

**Step 5: Verify no references to old marker system remain**

```bash
cd /Users/chris/Projects/hyper
grep -r "replace_markers" --include="*.py" --include="*.rs" .
grep -r "‹ESCAPE" --include="*.py" --include="*.rs" .
grep -r "‹CLASS" --include="*.py" --include="*.rs" .
grep -r "‹BOOL" --include="*.py" --include="*.rs" .
```

Expected: No matches (except possibly in the design doc's "before" examples or git history).
