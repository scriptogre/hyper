# Component language implementation plan

**Status:** Design complete for block syntax. Implementation not started.

This temporary checklist tracks delivery of the durable specifications:

- [Template language](../design/templates.md)
- [Template compiler](templates.md)
- [Error messages](../standards/error-messages.md)

Delete this file when every release gate is complete. Keep lasting behavior and architecture in the linked documents.

The durable docs describe the approved target, including behavior that has not landed yet. Conformance tests prove the implementation reaches that target.

## Confirmed direction

### Product

- The Python package and runtime are `hyperhtml`.
- Templates remain backend-neutral and separate from a future `hyperapi` package.
- Applications have no build step and no generated `.py` files.
- Installing the wheel activates `.hyper` imports automatically. Users do not call an installer or bootstrap import.
- Compilation runs in process through PyO3 with no subprocess fallback.
- Python wheels use the Python 3.10+ stable ABI.
- File generation is not a public CLI. A private IDE transport may remain where the JetBrains plugin needs one.

### Files and imports

- A file with top-level template output is an implicit component named after the file.
- `from app.components import Greeting` returns the real component function.
- An implicit component is not also a normal Python submodule.
- A declaration-only file is a component library and may export several components, functions, classes, and constants.
- Libraries use normal module imports such as `from app.components.controls import Button`.
- Compiler metadata, not naming conventions or generated-source inspection, determines the file mode and exports.
- Import order must never change a component function into a module.

### Components

- `@render_here` is coming soon and is not part of the alpha.
- Generated render functions use the sole runtime decorator `@component`; there is no `@html` alias.
- `@component` returns a callable `Component` that buffers calls and exposes raw chunks through `.stream()`.
- Generated parents use `@component(subcomponents=[Header, Footer])`; each child becomes a read-only attribute under its own `__name__`.
- `@render_here` exports a declared subcomponent and renders it at that exact source position.
- `@render_here(...)` accepts keyword arguments; explicit arguments override automatic same-name binding.
- Automatic binding uses matching props, locals, and loop variables available at that source position.
- Positional `@render_here(...)` arguments are invalid.
- Missing required values and arguments absent from the component signature are compile errors.
- Declared components never close over parent render state.
- `component Name(...):` declares a component. A normal `def` remains a normal Python function.
- Implicit and declared component props are keyword-only.
- Declared components with named props must write the leading `*` explicitly.
- Declared components reject positional props, `/`, and `*args`; `**attrs` remains valid.
- Slot values are keyword-only component arguments and appear in the public signature.
- The default slot uses the reserved `content` argument; named slots use their source names.
- A `.hyper` prop cannot use `content` or share a named slot's name; these collisions are compile errors.
- Transparent `<>...</>` fragments group output but do not declare a renderable component.
- HTML is valid in implicit rendering code and component declarations, not normal Python functions.
- Component code supports normal Python statements and control flow.
- `async component` is explicit. Implicit components infer async from `await`, `async for`, or `async with` in their own rendering scope.
- Bare `return` stops rendering.
- `return value` and explicit `yield` are invalid only in the active component scope.
- Nested normal Python functions retain normal return and yield behavior.

### Blocks and contexts

- Indented compound statements require indentation and an aligned `end`.
- Inner statements close before outer statements.
- Each compound statement owns one `end`; `elif`, `else`, `except`, `finally`, and `case` own none.
- A `match` has one `end`, regardless of its number of cases.
- Structural indentation uses spaces. Tabs are rejected.
- `end` may have a trailing Python comment.
- Statements begin only at the first non-whitespace position of a logical line.
- Statements may appear among HTML children but not inside tags, expressions, Python continuations, or raw content.
- Same-line forms are supported as optional shorthand and are not the primary documented style.
- Same-line content finishes on that logical line and does not use `end`.
- Same-line content selects Python or template context once. The contexts cannot mix.
- Python semicolons separate Python simple statements. Template semicolons remain content.
- Transparent `<>...</>` fragments group output without a wrapper and make bare template text explicit.
- Literal text resembling control flow uses a transparent fragment or template expression.

### Parser and runtime

- One indentation-aware parser replaces separate header, body, and case termination paths.
- Structural indentation is separate from rendered whitespace.
- Outer-colon detection ignores strings, annotations, patterns, dictionaries, slices, lambdas, and nested brackets.
- Automatic activation remains lazy, avoids recursive package scans, and does not load the runtime or optional dependencies until needed.
- Compiler errors retain source ranges, related labels, and help across PyO3.
- Generated source uses a stable filename and `linecache`; tracebacks do not show `<string>`.
- Integrations use the public loader without permanently mutating `sys.path`.

### Documentation and tests

- The language guide uses progressive disclosure and leads with indented forms.
- Same-line forms get one brief note. Their edge cases stay in tests and implementation docs.
- Durable design and implementation docs describe the target behavior.
- Exact errors live in expected-output tests; durable docs show representative corrections.
- Work proceeds in red-green slices, with the full affected test set green for every commit.

## Working method

For each behavior:

1. Add a focused test.
2. Confirm it fails for the intended reason.
3. Implement or refactor until it passes.
4. Run the focused tests.
5. Run all Rust, Python, and affected JetBrains tests.
6. Commit the test and implementation together.

Expected-output updates follow the repository review and approval workflow.

## Test matrix

### Lexical contexts

- [ ] Statements begin only at logical-line boundaries.
- [ ] Control-looking content in strings, Python continuations, `<script>`, and `<style>` stays in context.
- [ ] Statements inside opening tags produce the dedicated error.
- [ ] Structural indentation and rendered whitespace have separate assertions.
- [ ] Component, element, expression, and fragment ranges remain complete.

### Same-line forms

Cover `if`, `for`, `while`, `with`, `case`, `def`, `component`, and supported async forms:

- [ ] Python simple statements.
- [ ] Semicolon-separated Python statements.
- [ ] Semicolons inside Python strings.
- [ ] Elements, components, fragments, and template expressions.
- [ ] Mixed Python and template rejection.
- [ ] Incomplete tags and fragments.
- [ ] Outer colons after annotations, patterns, dictionaries, slices, lambdas, strings, and brackets.
- [ ] Mixed same-line and indented branch clauses.
- [ ] No extra `end` after a complete single-line statement.

### Indented forms

Apply this matrix to every compound statement:

- [ ] Valid indentation and aligned `end`.
- [ ] Unindented content.
- [ ] Missing, over-indented, and under-indented `end`.
- [ ] Nested blocks closed in the wrong order.
- [ ] Blank lines and comments before content.
- [ ] Valid and invalid branch-clause alignment.
- [ ] One `end` for `match`; none for individual `case` clauses.
- [ ] Multiline Python continuations.
- [ ] Tabs and mixed indentation.
- [ ] Trailing comments after `end`.

### Component scopes

- [ ] Explicit and implicit components.
- [ ] Normal Python functions and classes.
- [ ] Arbitrary supported Python statements inside components.
- [ ] Bare component `return`.
- [ ] Rejected component `return value` and `yield`.
- [ ] Normal return and yield inside nested Python functions.
- [ ] Explicit and inferred async with correct scope boundaries.
- [ ] Slots, escaping, composition, and streaming.
- [ ] Generated Python syntax and source ranges.

### File modes and loading

- [ ] Plain HTML and props as implicit components.
- [ ] Declaration-only component libraries.
- [ ] Multiple exports from one library.
- [ ] Filename and implicit component name validation.
- [ ] Deterministic package-level component imports.
- [ ] Normal library imports.
- [ ] Corrective implicit-submodule errors.
- [ ] Import-order independence.
- [ ] Relative imports, regular packages, and namespace packages.
- [ ] Package initialization and user hooks.
- [ ] Concurrent imports, failure cleanup, caching, and introspection.
- [ ] Automatic wheel activation and lazy runtime loading.

### Activation and packaging

- [ ] Unrelated Python startup does not load `hyperhtml`, MarkupSafe, optional dependencies, or `_native`.
- [ ] Finder precedence preserves built-in, frozen, standard-library, and resolved `.py` behavior.
- [ ] No recursive package scans.
- [ ] Subprocess and multiprocessing activation.
- [ ] Broken native extension guidance.
- [ ] Wheel, source distribution, and editable-install smoke tests.
- [ ] Measured startup time and imported-module budgets.

### Diagnostics

- [ ] Filename, source spans, related labels, help, and source lines survive PyO3.
- [ ] Generated syntax errors use a stable filename.
- [ ] Runtime tracebacks never use `<string>`.
- [ ] Import errors preserve their causes.
- [ ] UTF-8 source and decoding errors are explicit.

## 1. Characterize lexical contexts

- [ ] Cover statement, HTML, tag, expression, Python continuation, and raw-content contexts.
- [ ] Separate structural indentation assertions from rendered whitespace assertions.
- [ ] Preserve complete element, component, expression, and source ranges.

Complete when the lexical-context section of the compiler conformance matrix is green without production behavior changes.

## 2. Unify block parsing

- [ ] Replace header, body, and case termination loops with shared indentation-aware parsing.
- [ ] Support same-line Python and template content.
- [ ] Lock same-line content to one context.
- [ ] Support semicolons according to the selected context.
- [ ] Require indentation and aligned `end` for content on following lines.
- [ ] Give each compound statement one `end`; give branch clauses none.
- [ ] Add transparent `<>...</>` fragments.
- [ ] Preserve source ranges and whitespace semantics.
- [ ] Add the same-line and indented conformance matrices.

Complete when every compound statement follows the durable block rules and all existing parser invariants pass.

## 3. Add explicit components

- [ ] Parse and lower `component` and `async component`.
- [ ] Keep normal `def` semantics independent of HTML discovery.
- [ ] Support Python statements, template output, slots, and composition in components.
- [ ] Allow bare `return`.
- [ ] Reject `return value` and explicit `yield` in the active component scope.
- [ ] Preserve normal return and yield behavior in nested Python scopes.
- [ ] Infer async for implicit components only from their own rendering scope.
- [ ] Add source-map and JetBrains coverage.

Complete when the component conformance matrix is green and all generated Python is syntax-checked.

## 4. Classify files

- [ ] Return structured file mode and export metadata from Rust.
- [ ] Expose metadata through PyO3.
- [ ] Classify top-level rendering files as implicit components.
- [ ] Classify declaration-only files as library modules.
- [ ] Validate implicit component names from filenames.
- [ ] Reject ambiguous file structures according to the resolved design.

Complete when every valid file has exactly one compiler-reported mode and the loader performs no source inference.

## 5. Make imports deterministic

- [ ] Return a real function for `from app.components import Greeting`.
- [ ] Load component libraries as normal modules.
- [ ] Reject implicit-component submodule imports with the documented correction.
- [ ] Prevent import order from changing attribute types.
- [ ] Support relative imports, regular packages, and namespace packages.
- [ ] Preserve package initialization and resolved user hooks.
- [ ] Cover concurrency, failure cleanup, caching, and introspection.

Complete when the file-mode and import conformance matrix passes in isolated subprocesses.

## 6. Make activation lightweight

- [ ] Install a minimal finder automatically from the wheel.
- [ ] Keep the runtime, MarkupSafe, optional dependencies, and native compiler lazy.
- [ ] Place the finder after built-in and frozen importers.
- [ ] Remove recursive package scans.
- [ ] Set and test startup time and module-load budgets.
- [ ] Cover subprocesses, multiprocessing, broken native extensions, and editable installs.

Complete when a clean wheel provides automatic imports within the measured startup budgets.

## 7. Preserve diagnostics

- [ ] Carry structured compiler errors through PyO3.
- [ ] Preserve filename, ranges, source, related labels, and help.
- [ ] Compile generated Python with a stable synthetic filename.
- [ ] Register generated source with `linecache`.
- [ ] Remove `<string>` from syntax errors and runtime tracebacks.
- [ ] Add exact expected errors for every documented correction.

Expected errors cover:

- implicit components imported as submodules;
- HTML inside normal Python functions;
- `await` inside non-async declared components;
- component `return value` and explicit `yield`;
- missing indentation and missing or misaligned `end`;
- inner blocks left open by an outer `end`;
- statements inside HTML tags;
- mixed Python and template content on one line;
- incomplete same-line tags and transparent fragments;
- invalid native compiler installations.

Every error shows the failing span and a copy-ready correction.

Complete when the packaging and diagnostics conformance matrix is green.

## 8. Stabilize integrations and packaging

- [ ] Make integrations use the public import path.
- [ ] Stop discovery from permanently mutating `sys.path`.
- [ ] Define orphan and namespace-package behavior.
- [ ] Install and smoke-test built wheels in CI.
- [ ] Test source distributions and editable contributor setup.
- [ ] Run supported-platform coverage.

Complete when applications and integrations share one loader implementation and all install paths have automated tests.

## 9. Remove file generation

- [ ] Remove the public generation command.
- [ ] Stop the JetBrains plugin from writing `.py` files.
- [ ] Give the private IDE compiler bridge a purpose-specific name and protocol.
- [ ] Remove CLI-only dependencies from the runtime compiler.
- [ ] Remove generated-file workflows from documentation and examples.

Complete when no public workflow generates Python files and Rust, Python, and JetBrains tests pass.

## Open decisions by phase

Resolve a decision before starting the phase that depends on it.

### Explicit components

- [ ] Decide whether explicit components in an implicit file are allowed and private.
- [x] Require explicit keyword-only component props and allow `**attrs`; expose slots as keyword-only arguments.
- [ ] Define component decorator order.
- [ ] Decide whether explicit components may be nested.
- [ ] Define sync and async component composition without losing stream chunk boundaries.

### File scopes, modes, and imports

- [ ] Define which header declarations are module-level and which run per render.
- [ ] Define whether `Final` values are constants, parameters, or another construct.
- [ ] Define which names from an implicit component file are externally importable.
- [ ] Decide whether bare top-level implicit component files are importable.
- [ ] Define `.py` and `.hyper` precedence.
- [ ] Define package `__getattr__` and `__dir__` composition.

### Block details

- [ ] Decide whether ordinary Python compounds in headers and library module scope use aligned `end` or Python dedentation. The alpha may retain the current `end` behavior without making it permanent.
- [x] Use spaces for structural indentation and reject tabs.
- [x] Require transparent fragments or expressions for literal text that resembles control flow.
- [x] Allow a trailing Python comment after `end`.

### Activation

- [ ] Set startup time and imported-module budgets.
- [ ] Define the supported editable-install workflow.

## Documentation cleanup

Before implementation begins:

- [ ] Restructure `docs/implementation/templates.md` as a compiler contract rather than a second usage guide.
- [ ] Keep one source-to-Python example.
- [ ] Remove framework usage duplicated by the design guide and runtime docs.
- [ ] Organize the document by file classification, scopes, parsing, generation, loading, source maps, and errors.
- [ ] Mark unresolved semantics as plan decisions instead of choosing them in generated-code examples.

## Release gate

- [ ] Every required conformance matrix is green.
- [ ] Fresh wheels provide automatic zero-build imports.
- [ ] Import order cannot change runtime behavior.
- [ ] Compiler errors and runtime tracebacks preserve source context.
- [ ] No public CLI or generated `.py` workflow remains.
- [ ] Durable design and implementation docs match the shipped behavior.
- [ ] README and examples run unchanged in clean environments.
- [ ] All Rust, Python, packaging, and JetBrains tests pass.
