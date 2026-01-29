# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hyper is a Python template framework. Write templates in `.hyper` syntax, compile to type-safe Python code.

Monorepo with 3 components:
- **Rust transpiler** (`rust/transpiler/`) — Compiles `.hyper` → `.py`
- **Python runtime** (`python/`) — Runtime helpers, CLI, optional content collections
- **JetBrains plugin** (`editors/jetbrains-plugin/`) — IDE support via language injection

## Build and Test Commands

```bash
# Transpiler
just build transpiler          # Release build
just test transpiler           # Run all tests (or: cd rust && cargo test)
just compile <files-or-dirs>   # Build + run in one step (debug, suppresses warnings)
just test-update               # Accept all pending snapshots (cargo insta test --accept)
just test-review               # Review snapshots interactively (cargo insta review)

# Plugin
just build plugin              # Build transpiler + bundle binary + build plugin
just run plugin                # Launch sandbox IDE
just test plugin               # Run plugin tests

# Python
uv sync                        # Setup workspace
pytest                         # Run Python tests

# Generate .py from .hyper (uses release binary)
just generate <files>
```

The `compile` recipe accepts directories — it walks them for `.hyper` files.

## Transpiler Architecture

Three-stage pipeline in `rust/transpiler/src/lib.rs`:

1. **Parser** (`parser/`) — `tokenizer.rs` lexes into tokens, `tree_builder.rs` builds AST. Line-based tokenizer with `after_structural` flag for comment detection, `is_control_flow()` requiring trailing `:` to distinguish Python keywords from content text.

2. **Transformer** (`transform/`) — Visitor-pattern plugins (`HelperDetectionPlugin`, `AsyncDetectionPlugin`, `SlotDetectionPlugin`) analyze the AST and produce `TransformMetadata`.

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

**Snapshot tests** using `insta` crate (`rust/transpiler/tests/golden_tests.rs`):
- Test files: `rust/transpiler/tests/<category>/<name>.hyper`
- Snapshots: `rust/transpiler/snapshots/<category>@<name>@<suffix>.snap`
- Three test functions: `test_transpile_output`, `test_transpile_injections`, `test_transpile_errors`
- Error tests live in `tests/errors/` and are auto-detected

**Snapshot workflow:**
```bash
cargo test                      # Run tests, see failures
cargo insta accept              # Accept all pending snapshots
cargo insta review              # Review interactively
just test-update                # Shortcut for accept-all
```

## CLI Modes (`main.rs`)

- `hyper generate <files|dirs>` — Compile to `.py` files, walks directories
- `hyper generate --stdin` — Read from stdin, write to stdout
- `hyper generate --json` — JSON output with source mappings
- `hyper generate --daemon` — Length-prefixed JSON protocol for IDE integration

## Gotchas

- Transpiler binary must be rebuilt and re-bundled for plugin changes: `just build`
- Plugin requires JDK 17+
- Snapshot tests use `insta` — never edit `.snap` files by hand, use `cargo insta accept`
- The tokenizer is line-based; multiline Python expressions (paren/bracket spanning lines) are a known limitation
- `is_control_flow()` uses trailing `:` heuristic — content text that starts with a Python keyword and ends with `:` inside an element is an edge case
