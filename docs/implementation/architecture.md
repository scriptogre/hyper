# Compiler architecture

Hyper is a lean Rust host for Python DSLs.

## Pipeline

```text
Source
  -> plugin syntax parsing and lowering
  -> Ruff Python AST
  -> plugin checks
  -> plugin transforms
  -> source-preserving Python generation
  -> compilation result
```

The syntax stage emits parseable Python, source mapping, and typed plugin-owned data.

The Ruff AST is the only whole-file tree. A dialect may use a private tree for its own regions.

## Core

Core owns:

- pipeline coordination
- Ruff parsing and AST
- deterministic plugin scheduling
- typed compilation context
- structured diagnostics
- Python generation
- source-map composition

Core knows nothing about HTML, components, slots, fragments, special attributes, file components, separators, optional block terminators, extensions, filesystem I/O, Python import hooks, or rendering runtimes.

A non-HTML DSL requires no core change.

## Ruff

Use `ruff_python_parser`, `ruff_python_ast`, `ruff_text_size`, `ruff_source_file`, and `lsp-types` directly.

Use `ruff_python_codegen` where suitable for generated fragments.

Pin all Ruff crates to one revision. Do not copy or vendor Ruff code or types unless an isolated proof finds no smaller supported solution.

## Plugins

Built-in and third-party plugins use the same interface. A preset is an ordered plugin list.

The HTML package supplies the standard Hyper preset.

A plugin may own any subset of syntax parsing and lowering, checks, and transforms.

Syntax plugins parse their own regions. They emit parseable Python, source mapping, and typed data for later stages.

Checks inspect the whole file without mutation and emit structured diagnostics. All checks finish before transforms start.

Transforms run in deterministic preset order. They may mutate, replace, insert, or remove Ruff AST nodes.

Core may combine compatible visitors into one traversal without changing plugin order or observable behavior.

Plugins share a typed compilation context. Values use Rust types as keys, never strings.

Typed node and range data also lives in the context. Ruff nodes receive no plugin-specific fields.

Preset construction validates dependencies and order. The validation API remains open.

## Generation

Generation preserves unchanged user Python byte-for-byte, except indentation required by generated enclosing code.

Comments and blank lines remain intact. A file without DSL syntax compiles to itself.

Use Ruff generation for compiler-made fragments where it fits. Do not assume Ruff generates the whole tree.

Every mapped source range has an exact generated range.

Generated fragments splice verbatim user slices into Ruff-printed calls so user source ranges remain exact. Whole-file source-preserving generation may use AST printing or ranged edits.

## Positions

Core uses Ruff byte `TextRange` values internally.

Core converts positions once with `ruff_source_file`. The caller chooses UTF-8, UTF-16, or UTF-32 coordinates.

Delivery renders diagnostics without reinterpreting positions.

## Result

A compilation result contains Python source, source-map segments, `lsp_types::Diagnostic` values, and typed plugin outputs required by adapters.

Python output is usable only when no error diagnostics exist.

LSP defines no source-map type, so source-map segments remain a small core type.

Exact result fields and the location of diagnostic help text remain code decisions.

## Reuse

### HTML

Source:

```hyper
name: str
---
<h1>Hello {name}</h1>
```

HTML plugins privately parse and lower their regions. Ruff parses the resulting Python.

Checks validate the file. Transforms create the function, yields, and escaping.

Generation emits Python and maps it to the original source.

### SQL

Illustrative source:

```python
def rows(limit):
    sql:
        SELECT * FROM events LIMIT {limit}
```

SQL plugins privately parse and lower the query. The same core parses the resulting Python.

Checks validate SQL and interpolation. Transforms produce a form such as:

```python
Query("SELECT * FROM events LIMIT ?", [limit])
```

Generation emits and maps the Python.

The SQL block and placeholder forms are illustrative, not chosen syntax.

## Packages

```text
outer delivery
    -> HTML package
        -> core
```

Core has no HTML, filesystem, Python runtime, or import-hook dependency.

The HTML package owns HTML syntax and features, the ordered Hyper preset, the string renderer, the optional FastHTML renderer, the `.hyper` convention, runtime glue, the import hook, and the IDE bridge.

CLI and PyO3 delivery may be a third crate, HTML features, or binaries. Exact crate names remain open.

## Implementation order

1. Prove syntax lowering preserves original ranges and supports existing language fixtures.
2. Prove whole-file source-preserving generation and exact generated ranges.
3. Introduce `ruff_source_file`, result, and `lsp_types::Diagnostic` types behind current behavior.
4. Move remaining built-in language features onto the shared plugin interface.
5. Separate generic core from HTML and delivery.

Each step preserves generated behavior, error and help locations, IDE injections, UTF encodings, CLI behavior, and PyO3 behavior unless deliberately changed and reviewed.

## Open mechanisms

- syntax placeholder and region binding
- whole-file source-preserving generation
- stable node identity after replacement
- preset dependency and order validation
- an optional shared indentation-aware scanner
- package names and an optional third delivery crate
