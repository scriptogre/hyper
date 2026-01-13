# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hyper is a Python framework for hypermedia-driven applications. The core concept: write templates in `.hyper` syntax, transpile to type-safe Python code.

**Architecture**: Monorepo with 3 main components:
1. **Rust transpiler** (`rust/transpiler/`) - Compiles `.hyper` → `.py` using a pipeline architecture
2. **Python runtime** (`python/`) - Runtime helpers, CLI, and optional content collections
3. **JetBrains plugin** (`editors/jetbrains-plugin/`) - IDE support via language injection

## Building and Testing

### Transpiler (Rust)

```bash
# Build transpiler binary
just build transpiler

# Run transpiler
just run transpiler <files>
# Or directly:
./rust/target/release/hyper generate <files>

# Test transpiler
just test transpiler
# Or:
cd rust && cargo test

# Update golden test snapshots
UPDATE_SNAPSHOTS=1 cargo test
```

### JetBrains Plugin

```bash
# Build plugin (builds transpiler, bundles binary, builds plugin)
just build plugin

# Run plugin in sandbox IDE
just run plugin

# Test plugin
just test plugin

# Install built plugin
# Output: editors/jetbrains-plugin/hyper-plugin.zip
```

The plugin bundles the transpiler binary at `src/main/resources/bin/hyper-darwin-arm64`. For other platforms, rebuild and update the path in `HyperTranspilerService.kt`.

### Python Runtime

```bash
# Setup workspace (from repo root)
uv sync

# Run Python tests
pytest

# Generate .py from .hyper files
just generate <files>
```

## Transpiler Pipeline Architecture

The transpiler uses a clean 3-stage pipeline in `rust/transpiler/src/lib.rs`:

1. **Parser** (`parser/`) - Tokenizes `.hyper` source and builds AST
   - `tokenizer.rs` - Lexical analysis
   - `tree_builder.rs` - Constructs AST from tokens
   - `positions.rs` - Tracks source positions for IDE integration

2. **Transformer** (`transform/`) - Plugin-based AST transformations
   - Visitor pattern for traversing/modifying AST
   - Standard plugins: `HelperDetectionPlugin`, `AsyncDetectionPlugin`, `SlotDetectionPlugin`
   - Produces `TransformMetadata` for code generation

3. **Generator** (`generate/`) - AST → Python code
   - `python.rs` - Main code generator (combines nodes into f-strings, emits control flow)
   - `injection_analyzer.rs` - Analyzes code for IDE language injection ranges
   - `output.rs` - Manages generated code with position tracking

**Key insight**: The generator combines consecutive text/expression/element nodes into single f-strings, but emits control flow (if/for/match) as separate statements.

## Testing Strategy

**Golden tests** (`rust/transpiler/tests/golden_tests.rs`):
- Each `.hyper` file can have 3 companion files:
  - `.expected.py` - Expected Python output
  - `.expected.injections.json` - Expected IDE injection ranges
  - `.expected.txt` - Expected error message
- Update snapshots: `UPDATE_SNAPSHOTS=1 cargo test`

**Test locations**:
- `rust/transpiler/tests/` - Transpiler unit/golden tests
- `playground/` - Example `.hyper` files for manual testing

## IDE Integration

The JetBrains plugin (`editors/jetbrains-plugin/`) provides IDE features through **language injection**:

1. Transpiler generates virtual Python code for each `.hyper` file
2. Plugin injects this Python into IDE's language server
3. IDE provides completion, go-to-definition, type checking
4. Position mapping handled via `injection_analyzer.rs` and `HyperLanguageInjector.kt`

**Key files**:
- `HyperTranspilerService.kt` - Manages transpiler binary, caches results
- `HyperLanguageInjector.kt` - Injects Python/HTML into IDE
- `HyperFileListener.kt` - Auto-generates `.py` on save
- `HyperInjectionViewerToolWindow.kt` - Debug view of transpiled code
- `Hyper.bnf`/`Hyper.flex` - Grammar definitions

## Workspace Structure

```
python/               Python packages (uv workspace)
  hyper/             Runtime + CLI
rust/
  transpiler/        Core transpiler (Parser → Transform → Generate)
editors/
  jetbrains-plugin/  IDE support
playground/          Example .hyper files
docs/design/         Architecture docs
```

## Package Management

- **Python**: Uses `uv` for workspace management
  - Workspace root: `/pyproject.toml`
  - Package: `python/pyproject.toml`
  - Install with content collections: `pip install hyper[content]`

- **Rust**: Standard Cargo workspace
  - Single package: `rust/transpiler/Cargo.toml`

## Common Gotchas

- The transpiler binary must be rebuilt and bundled when making changes: `just build`
- Plugin requires JDK 17+
- Golden tests require exact string matches - use `UPDATE_SNAPSHOTS=1` when intentionally changing output
- IDE injection requires valid Python syntax - malformed templates won't get IDE features