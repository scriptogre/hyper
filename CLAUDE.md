# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hyper is a Python template framework. Write templates in `.hyper` syntax, compile to type-safe Python code.

Monorepo with 3 components:
- **Rust transpiler** (`rust/`) — Compiles `.hyper` → `.py`
- **Python runtime** (`python/`) — Runtime helpers, CLI, optional content collections
- **JetBrains plugin** (`editors/jetbrains/`) — IDE support via language injection

## Build and Test Commands

```bash
# Transpiler
just build                     # Release build
just test                      # Run all tests (or: cd rust && cargo test)
just test-accept               # Regenerate all .expected.* files from current output
just test-accept basic         # Regenerate only files matching "basic"

# Plugin
just build-plugin              # Build transpiler + bundle binary + build plugin
just run-plugin                # Launch sandbox IDE
just test-plugin               # Run plugin tests

# Python
uv sync                        # Setup workspace
pytest                         # Run Python tests

# Generate .py from .hyper (uses release binary)
just generate <files>
```

The `compile` recipe accepts directories — it walks them for `.hyper` files.

## Transpiler Architecture

Three-stage pipeline in `rust/src/lib.rs`:

1. **Parser** (`parser/`) — `tokenizer.rs` lexes into tokens, `tree_builder.rs` builds AST. Line-based tokenizer with `after_structural` flag for comment detection, `is_control_flow()` requiring trailing `:` to distinguish Python keywords from content text.

2. **Transformer** (`transform/`) — Visitor-pattern plugins (`HelperDetectionPlugin`, `AsyncDetectionPlugin`, `SlotDetectionPlugin`, `SpreadDetectionPlugin`) analyze the AST and produce `TransformMetadata`.

3. **Generator** (`generate/`) — `python.rs` combines consecutive text/expression/element nodes into single f-strings, emits control flow as separate statements. `injection_analyzer.rs` produces IDE language injection ranges. `output.rs` tracks source positions with UTF-16 mapping.

### Compile-time HTML validation (`html.rs` + `tree_builder.rs`)

The parser validates HTML at parse time:
- **Void elements** — `<br>`, `<img>`, etc. cannot have children or closing tags
- **Duplicate attributes** — Same attribute name twice on one element
- **Invalid nesting** — Block elements inside `<p>`, nested interactive elements (`<a>` inside `<button>`)

Validation uses an `element_stack` in `TreeBuilder` for parent context.

### Error system (`error.rs`)

`ParseError` renders with ANSI colors when stderr is a TTY:
- `render()` — plain text (for piped output, tests)
- `render_color()` — colored (red errors, blue line numbers, cyan related spans, yellow help)
- `Display` impl outputs just the message (no redundant kind prefix)
- Errors support `related_span` with custom `related_label` and multiline `help` text

## Testing

**Expected output tests** (`rust/tests/expected_tests.rs`):
- Test files: `rust/tests/<category>/<name>.hyper`
- Expected output: `<name>.expected.py` (compiled Python), `<name>.expected.json` (injection ranges/injections), `<name>.expected.err` (error messages)
- Error tests live in `tests/errors/` and are auto-detected

**Expected output workflow:**
```bash
cargo test                              # Run tests, see failures
cargo run --bin accept_expected         # Regenerate all .expected.* files from current compiler output
cargo run --bin accept_expected basic   # Regenerate only files matching "basic"
```

**IMPORTANT — expected output review:** `cargo run --bin accept_expected` blindly stamps the compiler's current output as correct. Never run it after a change without manually reviewing the diffs (`git diff`) to confirm the new output is actually what you expect. A bug in the compiler will silently become the blessed expected output otherwise. When changing parser or codegen logic, always spot-check at least the directly affected `.expected.py` and `.expected.json` files before considering the change done.

**CRITICAL — injection range validation:** Every Python injection range `source[start:end]` must extract to meaningful text from the source file (not mid-word garbage). After accepting expected output, verify that source positions in `.expected.json` files map to the correct source text. The test suite includes semantic validation (range text extraction checks) — if these fail, the ranges are wrong, do NOT blindly accept. Common mistakes: off-by-one in span calculations, stale expected files accepted without review, substring-matching tests that pass accidentally.

**Invariant tests** (`rust/tests/invariants/`):
- Property-based checks that run across ALL `.hyper` test files automatically
- Each module validates one structural invariant (roundtrip, monotonicity, bounds, html_completeness, etc.)
- New invariants go in their own module file under `invariants/`
- Adding a new `.hyper` test file automatically gets invariant coverage with zero extra work

**Kitchen sink smoke test** (`tests/basic/kitchen_sink.hyper`):
- Exercises every syntax construct in one file (elements, components, slots, control flow, decorators, attributes, expressions, comments)
- After any injection change, open this file in JetBrains and visually verify highlighting
- All 8 invariants run against it automatically

## CLI Modes (`main.rs`)

- `hyper generate <files|dirs>` — Compile to `.py` files, walks directories
- `hyper generate --stdin` — Read from stdin, write to stdout
- `hyper generate --json` — JSON output with source mappings
- `hyper generate --daemon` — Length-prefixed JSON protocol for IDE integration

## Gotchas

- Transpiler binary must be rebuilt and re-bundled for plugin changes: `just build`
- Plugin requires JDK 17+
- Expected output files (`.expected.py`, `.expected.json`, `.expected.err`) are managed by `cargo run --bin accept_expected` — never edit them by hand
- The tokenizer is line-based; multiline HTML tags (attributes spanning lines) are a known limitation. Multiline Python expressions work via bracket depth tracking.
- `is_control_flow()` uses trailing `:` heuristic — content text that starts with a Python keyword and ends with `:` inside an element is an edge case

## Bug Fix Workflow

When the user reports a bug, visual issue, or incorrect behavior — especially phrases like "not highlighted", "looks wrong", "should be X but is Y", "missing", "broken" — invoke the `/red-green-fix` skill before making any code changes. Never fix a bug without first writing a failing test that captures the expected behavior.

