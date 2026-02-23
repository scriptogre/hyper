# IDE Tooling Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the JetBrains plugin reliably provide Python intelligence, HTML intelligence, and error display in `.hyper` files.

**Architecture:** The transpiler generates injection ranges (source→compiled position mappings). The JetBrains plugin uses these to inject Python and HTML language support into `.hyper` files via `MultiHostInjector`. Three phases: fix Python injection, add HTML injection, add error display.

**Tech Stack:** Rust (transpiler), Kotlin (JetBrains plugin), `cargo test` + `just build plugin` + `just run plugin`

---

## Phase 0: Rebuild and Verify Baseline

### Task 1: Rebuild transpiler binary and bundle in plugin

The bundled binary is stale (still has old marker system). Rebuild and verify.

**Step 1:** Build the release transpiler binary:
```bash
just build transpiler
```

**Step 2:** Copy to plugin resources:
```bash
cp rust/target/release/hyper editors/jetbrains-plugin/src/main/resources/bin/hyper-darwin-arm64
```

**Step 3:** Build the plugin:
```bash
just build plugin
```

**Step 4:** Run the sandbox IDE:
```bash
just run plugin
```

**Step 5:** Open a `.hyper` file, check the Hyper Inspector "Python" tab — verify `escape()` calls appear instead of `‹ESCAPE:` markers. Verify Ctrl+Click on variables works.

**Step 6:** Commit:
```bash
git add editors/jetbrains-plugin/src/main/resources/bin/hyper-darwin-arm64
git commit -m "Rebuild transpiler binary for plugin"
```

---

## Phase 1: Fix Python Injection

### Task 2: Audit and fix Python injection ranges

The injection mechanism works but is "inconsistent." We need to verify that ALL Python expressions generate correct ranges, and debug any that don't.

**Files:**
- `rust/transpiler/src/generate/python.rs` (range creation during codegen)
- `rust/transpiler/tests/injection_tests.rs` (injection test suite)

**Step 1:** Create a comprehensive test `.hyper` file that exercises all expression contexts:
```hyper
name: str
count: int = 0
items: list[str] = []
is_active: bool = False
---
<h1>Hello {name}</h1>
<p>{count} items</p>
if is_active:
    <span class="active">Yes</span>
end
for item in items:
    <li>{item}</li>
end
```

Run the transpiler with `--json --injection` and verify every expression (`name`, `count`, `is_active`, `items`, `item`) has a corresponding injection range.

**Step 2:** Check that Ctrl+Click works for EACH expression in the sandbox IDE. Document any that fail.

**Step 3:** For each failing injection, trace through `python.rs` to find where the range should be created. Fix the range tracking.

**Step 4:** Add injection tests for any missing cases.

**Step 5:** Rebuild binary, bundle, test in IDE.

### Task 3: Fix parameter injection for frontmatter props

Currently parameters have `needs_injection: true` but only in the function signature context. Verify that typing a prop name above `---` gets Python intelligence (type annotations should resolve).

---

## Phase 2: Add HTML Injection Ranges

### Task 4: Generate HTML injection ranges in the transpiler

Currently `RangeType::Html` exists but is never created. We need to emit HTML ranges for tag content.

**Files:**
- `rust/transpiler/src/generate/python.rs` (add HTML range creation)
- `rust/transpiler/src/generate/output.rs` (already supports Html type)

**Key insight from the deleted branch:** HTML injections should cover:
- Opening tags: `<div class="foo">` (the whole tag including attributes)
- Closing tags: `</div>`
- NOT the content between tags (that's either Python expressions or more HTML)
- NOT control flow lines (`if`, `for`, `end`)

**Approach:**
Each `Element` node in the AST has source position info. During codegen, when we emit an element's opening tag, also emit an HTML range mapping the source tag position to the compiled f-string position.

The compiled output has HTML embedded in f-strings: `yield f"""<div class="foo">"""`. The HTML is literal text in the f-string. The injection range should cover just the HTML portions (not the `{escape(expr)}` Python expressions within them).

**For the IDE virtual HTML file:**
The prefix/suffix mechanism concatenates all HTML ranges into one virtual file. The IDE provides HTML intelligence on this virtual file, which maps back to source positions.

**Step 1:** In `python.rs`, when emitting element opening/closing tags, add `RangeType::Html` ranges with `needs_injection: true`.

**Step 2:** The ranges should cover the static HTML parts only. Where a tag has dynamic attributes like `{render_attr("disabled", expr)}`, the HTML range should stop before and resume after the Python expression.

**Step 3:** Add tests in `injection_tests.rs` for HTML ranges.

**Step 4:** Rebuild binary, bundle, test in IDE — verify HTML tags get colored and tag completion works.

### Task 5: Handle non-overlapping HTML and Python ranges

HTML and Python injections must not overlap. The `compute_injections()` function already processes each type separately, so this should work. But verify with a complex file that has mixed HTML and Python.

---

## Phase 3: Error Display

### Task 6: Surface transpiler errors in the IDE

When the transpiler returns an error (unclosed tag, invalid nesting), display it in the editor.

**Files:**
- `editors/jetbrains-plugin/src/main/kotlin/com/hyper/plugin/HyperLanguageInjector.kt`
- May need a new `HyperAnnotator.kt` or `HyperExternalAnnotator.kt`

**Approach:** Use JetBrains' `ExternalAnnotator` pattern:
1. The annotator calls the transpiler (reusing `HyperTranspilerService`)
2. If the transpiler returns an error with source position, create an annotation at that position
3. Display as red squiggly underline with the error message as tooltip

**Step 1:** Add error position info to the transpiler's JSON error output. Currently errors have line/column info — verify they're included in the JSON response.

**Step 2:** Create `HyperExternalAnnotator.kt` that:
- Calls `transpilerService.transpile()` in the background
- If error, creates `HighlightInfo` at the error position
- If success, clears previous errors

**Step 3:** Register in `plugin.xml`:
```xml
<externalAnnotator language="Hyper" implementationClass="com.hyper.plugin.HyperExternalAnnotator"/>
```

**Step 4:** Test with intentionally broken `.hyper` files.

---

## Testing Strategy

- **Transpiler side:** Injection tests in `injection_tests.rs` — verify range positions are correct
- **Plugin side:** Manual testing in sandbox IDE via `just run plugin`
- **Debug tool:** Use the existing Hyper Inspector tool window tabs (Python, Ranges, Injections) to diagnose issues

## What We're NOT Doing (YAGNI)

- Code formatting for .hyper files
- Custom autocomplete beyond what Python/HTML injection provides
- Rename refactoring
- Code folding
- Live templates/snippets
- Run configurations
- Cross-platform binary bundling (macOS ARM64 only for now)
