# Hyper Transpiler Migration Plan

This document captures architectural decisions and implementation priorities for the Rust transpiler migration.

## Context

We're mid-migration (`rust/transpiler/` is the new implementation). This is an opportunity to adopt patterns from mature template compilers like minijinja while the codebase is in flux.

**Key insight:** Hyper's extensibility is about *code generation*, not user-defined syntax. Features like fragments and LiveView-style reactivity require sophisticated output generation, not parser plugins.

## Architecture Decisions

### What We're Adopting

| Pattern | Source | Rationale |
|---------|--------|-----------|
| Rich error messages with spans | minijinja | Developer experience is critical for a template language |
| Insta snapshot testing | minijinja | Less test code, better review workflow, colored diffs |
| Multi-part generator output | - | Enables hoisting (fragments) and multi-file output (future JS generation) |

### What We're NOT Adopting

| Pattern | Why Skip |
|---------|----------|
| `Spanned<Box<T>>` enum optimization | Templates are small, premature optimization |
| Zero-copy `&'a str` AST | Adds lifetime complexity, marginal benefit |
| VM bytecode generation | We generate source code, not bytecode |
| Compile-time constant folding | Python handles this at import time |
| `#[cfg(feature)]` on AST variants | We don't have optional syntax, we have different output modes |

## Implementation Phases

### Phase 1: Error Infrastructure

**Goal:** Enable error messages like:
```
error: Unclosed element <div>
  --> components/Header.hyper:12:5
   |
12 |     <div class="header">
   |     ^^^^ opened here, never closed
   |
   = help: Add </div> to close this element
```

**Changes:**

1. **Expand `Span` struct** (already exists in tokenizer.rs, may need column tracking)
   ```rust
   pub struct Span {
       pub start_line: u32,
       pub start_col: u32,
       pub start_offset: u32,
       pub end_line: u32,
       pub end_col: u32,
       pub end_offset: u32,
   }
   ```

2. **Rich error types** in `src/error.rs`:
   ```rust
   pub struct ParseError {
       pub kind: ErrorKind,
       pub span: Span,
       pub related_span: Option<Span>,  // "opened here", "defined here"
       pub help: Option<String>,
   }

   pub enum ErrorKind {
       UnclosedElement { tag: String },
       UnclosedExpression,
       MismatchedTag { expected: String, found: String },
       InvalidForLoop { reason: String },
       UnexpectedToken { expected: String, found: String },
       UnexpectedEof,
       // Expand as needed
   }
   ```

3. **Error renderer** that formats with source context:
   - Extract source line from offset
   - Generate underline carets
   - Show related spans ("opened here")
   - Include help text

4. **Fix silent failures** in parser:
   - `parse_until_element_close` must error on EOF
   - `parse_until_component_close` must error on EOF
   - Track opening spans and pass to these functions

**Files to modify:**
- `src/error.rs` — New error types
- `src/parser/tokenizer.rs` — Ensure spans have column info
- `src/parser/tree_builder.rs` — Pass opening spans, return errors on unclosed elements
- `src/lib.rs` — Error rendering for CLI output

---

### Phase 2: Insta Migration

**Goal:** Replace ~200 lines of custom golden test code with ~50 lines using Insta.

**Changes:**

1. **Update Cargo.toml**:
   ```toml
   [dev-dependencies]
   insta = { version = "1.46", features = ["glob", "yaml"] }

   [profile.dev.package]
   insta.opt-level = 3
   similar.opt-level = 3
   ```

2. **Rewrite `tests/golden_tests.rs`**:
   ```rust
   use hyper_transpiler::{GenerateOptions, Pipeline};

   #[test]
   fn test_transpile() {
       insta::glob!("fixtures/**/*.hyper", |path| {
           let source = std::fs::read_to_string(path).unwrap();
           let name = path.file_stem().unwrap().to_str().unwrap();

           let mut pipeline = Pipeline::standard();
           let result = pipeline.compile(&source, &GenerateOptions {
               function_name: Some(name.to_string()),
               include_ranges: false,
           });

           match result {
               Ok(r) => insta::assert_snapshot!("output", r.code),
               Err(e) => insta::assert_snapshot!("error", format!("{}", e)),
           }
       });
   }

   #[test]
   fn test_injections() {
       insta::glob!("fixtures/**/*.hyper", |path| {
           let source = std::fs::read_to_string(path).unwrap();
           let name = path.file_stem().unwrap().to_str().unwrap();

           let mut pipeline = Pipeline::standard();
           if let Ok(result) = pipeline.compile(&source, &GenerateOptions {
               function_name: Some(name.to_string()),
               include_ranges: true,
           }) {
               insta::assert_yaml_snapshot!("injections", serde_json::json!({
                   "ranges": result.ranges,
                   "injections": result.injections,
               }));
           }
       });
   }
   ```

3. **Reorganize test files**:
   ```
   tests/
   ├── golden_tests.rs
   ├── fixtures/           # Renamed from category dirs
   │   ├── basic/
   │   ├── control_flow/
   │   └── ...
   └── snapshots/          # Auto-generated by Insta
   ```

4. **Migration steps**:
   - Run `cargo insta test --accept` to generate initial snapshots
   - Verify snapshots match old `.expected.*` files
   - Delete `.expected.py` and `.expected.injections.json` files
   - Update CI to use `cargo insta test`

**Workflow after migration:**
- `cargo insta test` — Run tests, collect pending changes
- `cargo insta review` — Interactive approve/reject
- CI sets `CI=true` to fail on pending snapshots

---

### Phase 3: Generator Output Refactor

**Goal:** Enable code hoisting for fragments without rewriting the generator.

**Changes:**

1. **New output structure** in `src/generate/mod.rs`:
   ```rust
   pub struct GeneratorOutput {
       /// Hoisted code (fragment definitions, helper functions)
       pub preamble: Vec<String>,
       /// The main component function
       pub main_function: String,
       /// Additional generated files (future: JS, CSS)
       pub auxiliary: Vec<AuxiliaryFile>,
   }

   pub struct AuxiliaryFile {
       pub filename: String,
       pub content: String,
   }
   ```

2. **Update `CompileResult`** in `src/lib.rs`:
   ```rust
   pub struct CompileResult {
       pub code: String,  // preamble.join("\n") + main_function
       pub preamble: Vec<String>,  // For tools that need them separately
       pub ranges: Vec<Range>,
       pub injections: Vec<Injection>,
   }
   ```

3. **Generator accumulates preamble** during walk:
   - When visiting `FragmentNode`, generate function and push to preamble
   - When emitting inline, just emit the function call

**Files to modify:**
- `src/generate/mod.rs` — New output types
- `src/generate/python.rs` — Accumulate preamble during generation
- `src/lib.rs` — Combine preamble + main in `CompileResult.code`

---

### Phase 4: Fragment Feature

**Goal:** Enable inline component definitions that hoist to top-level.

**Syntax:**
```hyper
name: str
---
fragment Greeting:
    <span class="greeting">Hello, {name}!</span>
end

<div>
    <{Greeting}/>
    <{Greeting}/>
</div>
```

**Generated Python:**
```python
def Greeting(__parent_scope__):
    name = __parent_scope__["name"]
    return f'<span class="greeting">Hello, {name}!</span>'

def template(name: str):
    __scope__ = {"name": name}
    return f'<div>{Greeting(__scope__)}{Greeting(__scope__)}</div>'
```

**Implementation:**

1. **AST node** (already have `FragmentNode` stub):
   ```rust
   pub struct FragmentNode {
       pub name: String,
       pub name_span: Span,
       pub children: Vec<Node>,
       pub span: Span,
   }
   ```

2. **Parser** — `tokenize_fragment_start` exists, complete `tree_builder` handling

3. **Transform** — Detect captured variables (variables used in fragment but defined in parent scope)

4. **Generator**:
   - Generate fragment as standalone function with scope parameter
   - Hoist to preamble
   - Emit call with current scope at usage site

---

### Phase 5: Multi-Target Generation (Future)

**Goal:** Generate Python + JavaScript for LiveView-style reactivity.

This phase is speculative. Document the architecture when requirements are clearer.

**Likely approach:**
```rust
pub trait CodeGenerator {
    fn generate(&self, ast: &[Node], metadata: &TransformMetadata) -> GeneratorOutput;
}

struct PythonGenerator;
struct LiveViewGenerator {
    python: PythonGenerator,
    // Also generates JS client code
}
```

**Considerations:**
- What JS framework/approach? (Alpine.js, custom, HTMX-style)
- How to express reactivity in .hyper syntax?
- WebSocket/SSE infrastructure on Python side?

---

## Open Questions

1. **Column tracking in spans** — Current `Position` has `byte` and `line` but not `col`. Need to add or compute from byte offset.

2. **Fragment scope capture** — How to detect which variables a fragment needs from parent scope? Walk the fragment AST looking for `Var` references not defined locally?

3. **Fragment naming conflicts** — What if user defines `fragment Foo` and also imports a component named `Foo`?

4. **Error recovery** — Should parser attempt to continue after errors (like tree-sitter) or fail fast? Fail fast is simpler and probably fine for a transpiler.

---

## References

- [minijinja](https://github.com/mitsuhiko/minijinja) — Error handling patterns, testing approach
- [Insta](https://insta.rs/) — Snapshot testing
- [Phoenix LiveView](https://hexdocs.pm/phoenix_live_view) — Inspiration for reactivity model
