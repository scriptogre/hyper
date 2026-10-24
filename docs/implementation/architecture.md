# Compiler architecture and migration

This is Hyper's only compiler architecture plan.

Code and tests describe the current implementation.
This file describes the intended architecture and the path to it.

## Next

Run the two proof tests in this file.
Do not migrate production code until both pass.

## Mental model

```text
Plugins recognize their syntax.
Ruff parses the remaining Python.
Plugins check and transform the Ruff tree.
Ruff prints Python.
Hyper returns the Python, source map, and diagnostics.
```

## Goal

Hyper becomes a lean Rust host for Python DSLs.

The generic compiler owns Python parsing, plugin coordination, diagnostics, and source maps.
It knows nothing about:

- HTML, components, slots, fragments, or special attributes
- implicit file components, `---`, or optional `end`
- the `.hyper` extension, file writing, or the Python import hook

Those features belong to plugins or outer packages.
A non-HTML Python DSL must not require changes to the generic compiler.

## Today

At commit `cba077a`, `rust/src` contains 10,759 lines.

```text
Source -> Parse -> Lower -> Plugins -> Generate -> UTF-16 conversion
```

| Code | Lines | Job |
|---|---:|---|
| `parse/tokenizer.rs` | 2,983 | Tokenizes mixed Python and HTML |
| `parse/tree_builder.rs` | 1,358 | Builds Hyper's custom tree |
| `generate/python.rs` | 1,681 | Emits Python and tracks ranges |
| `error.rs` | 1,038 | Defines and prints compiler errors |

Current constraints:

- `tree-sitter-python` is used by the tokenizer and component-signature parser.
- The central AST knows HTML, components, slots, fragments, and Python control flow.
- Function plugins each walk the tree separately.
- `FileComponent` and `Components` bypass the function-level plugin trait.
- JetBrains needs source and compiled ranges for Python and HTML injections.
- The Python import hook needs generated code and the component name, not IDE ranges.
- Main has no Ruff dependency.

## Intended system

```text
Source
  -> plugin-owned syntax handling
  -> ruff_python_ast
  -> plugin checks
  -> plugin transforms
  -> ruff_python_codegen
  -> Python text + source map + diagnostics
```

### Reuse

- `ruff_python_parser`: Python parsing
- `ruff_python_ast`: Python syntax tree
- `ruff_python_codegen`: Python generation
- `ruff_text_size` and `ruff_source_file`: source locations
- `lsp-types`: diagnostics and positions

Pin all Ruff crates to one revision.
Do not copy Ruff types into Hyper.

LSP does not define source maps, so segments remain a Hyper type.

### Plugins

Plugins run in a deterministic order.
A standard HTML preset configures the built-in Hyper plugins.

Use these terms:

- `context`: shared compilation state and typed plugin data
- `check`: read-only inspection and diagnostic reporting
- `transform`: mutation, replacement, or removal of syntax-tree nodes

A plugin may inspect the whole file before transforms begin.
Compatible checks and transforms share traversals instead of walking once per plugin.

Shared plugin data is typed.
Core nodes do not gain plugin-specific fields.
Plugins do not exchange data through string keys.

Built-in and third-party plugins use the same interface.
Core does not gain a special extension point only for `end`.

Exact Rust APIs live in code and rustdoc.

### Output

Compilation returns one value containing:

- generated Python text
- source-to-generated segments
- LSP-shaped diagnostics

File writing and execution consume that value outside the compiler.

The HTML package owns the standard `.hyper` preset, runtime, and import hook.
String output is its first rendering strategy.
FastHTML is a later HTML rendering strategy, not a generic compiler concern.

## Proofs

The two main simplifications are hypotheses.
Test them in isolation before changing production code.

### 1. Parse through Ruff

Test whether plugins can replace their syntax with valid Python before Ruff parses the file.

Cover:

- nested and multiline HTML
- `<br>` and `<>...</>`
- components, slots, shorthand, and spreads
- Python control flow between tags
- `end` and `---`
- `a < b` beside `<div>`
- `<` and braces inside strings, comments, scripts, and styles
- multibyte text and expressions

Pass when valid fixtures parse and all source, error, and IDE ranges remain exact in bytes and UTF-16.

Same-length masking is one candidate, not a requirement.
If it fails, choose another pre-Python representation.

### 2. Recover generated ranges

Test this candidate:

1. Print the transformed Ruff tree.
2. Parse the generated Python with Ruff.
3. Match the printed tree to the parsed tree.
4. Recover compiled ranges for JetBrains.

Cover nested helper calls, parentheses, multiline expressions, decorators, parameters, control flow, and one-to-many transforms.

Pass when every mapped source slice and generated slice satisfies the current JetBrains injection contract.

If structural matching fails, test marker substitution or an upstream Ruff hook.
Do not vendor Ruff's generator unless smaller approaches fail.

## Work

Each row is one focused session or mergeable change.

| # | Goal | Done when |
|---:|---|---|
| 1 | Prove the parser boundary | Proof 1 passes without changing production code |
| 2 | Prove generated-range recovery | Proof 2 selects a reliable mapping mechanism without changing production code |
| 3 | Establish Ruff | Ruff is pinned; `plugins/components.rs` no longer uses tree-sitter; output is unchanged |
| 4 | Add Ruff generation | The proven printer and range mapper pass isolated tests; the old generator remains active |
| 5 | Replace parsing | The proven parser feeds the existing AST; tokenizer, tree builder, and tree-sitter are deleted |
| 6 | Replace generation | Ruff AST and codegen replace the local Python AST and string generator |
| 7 | Unify plugins | Ordered registration, typed context, checks, and transforms replace special and per-plugin walks |
| 8 | Standardize diagnostics | LSP types replace superseded error/location structures without changing CLI or PyO3 behavior |
| 9 | Separate delivery | File handling, imports, the HTML preset, and future FastHTML rendering live outside the generic compiler |

Delete the Hyper-specific AST and lowering layer only after no plugin depends on them.
Review every expected-output change before accepting it.

## Invariants

Every production change preserves:

- generated Python behavior
- component and slot behavior
- error locations and help
- Python and HTML IDE injections
- UTF-16 source and compiled ranges
- CLI and PyO3 behavior until an explicit migration changes them

Run Rust and Python tests for every step.
Run affected JetBrains tests for source-map or protocol changes.

## Open decisions

These do not block the two proofs:

- neutral names for stages before the Ruff Python tree
- the exact typed `context` API
- how plugin ordering and dependencies are validated
- how one plugin exposes extension points to another
- how component-specific metadata leaves the generic compiler
- when to split the generic compiler and HTML package into separate crates

## Size

The parser, tree builder, generator, and local Python AST contain about 6,000 lines.
Most may disappear if both proofs pass.

A final Rust total near 2,500 to 4,000 lines is an estimate, not a requirement.
Correct behavior and clear boundaries take priority.

## Documentation

- `README.md`: public introduction and quickstart
- `ROADMAP.md`: releases, backlog, and known user-visible bugs
- `docs/design/templates.md`: `.hyper` language behavior
- `docs/implementation/templates.md`: current compiler behavior
- `docs/implementation/architecture.md`: intended architecture and migration

No other file contains a compiler architecture plan.
