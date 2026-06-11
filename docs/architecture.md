# Compiler architecture

How the Hyper transpiler is structured: the pipeline, what each plugin is allowed
to do, and which concern lives where.

## Status

This is the target design. The compiler is migrating to it in stages (see
"Migration"). Until they land, the "lives today" column in the inventory shows
where each concern currently sits.

## Prior art

The design follows the mainstream template-compiler spine, confirmed against
Svelte 5, Vue 3, and Babel:

- All three are **Parse, then ordered plugins over one shared mutable AST, then
  Generate**. Plugins walk nodes with enter/exit and may mutate, replace, or
  remove them. This is exactly our existing `Transformer`.
- Svelte keeps a read-only **Analyze** step that produces a separate analysis
  object instead of mutating. That is our `TransformMetadata`.
- Vue's **directiveTransforms** are keyed by directive name and translate a raw
  directive into props. That is our generic brace transformed by pattern-keyed
  plugins (below).

What we deliberately do NOT copy: a heavyweight multi-phase framework. None of the
three has one. They have a single ordered plugin list. So do we.

## The pipeline

Two stages are irreducible: **Parse** (source to AST) and **Generate** (AST to
Python). The rest hook around them.

```
Lex  ->  PARSE  ->  Plugins  ->  GENERATE  ->  Map
(pre)    (core)     (transform,   (core)        (post)
                     scan, guard)
```

- **Lex**: text becomes tokens. Lexical only, no meaning.
- **Parse**: tokens become a surface AST plus structural checks (unclosed tags,
  void elements with children, duplicate attributes). These checks belong here
  because they fall out of building the tree, as in Svelte and Vue.
- **Plugins**: one ordered list of plugins over the shared AST. The middle of
  the compiler. Everything pluggable lives here.
- **Generate**: AST plus metadata becomes Python plus raw source positions.
- **Map**: raw positions become IDE injection ranges, in UTF-16.

## Plugin roles

Plugins run in registration order (like Babel and Vue). Each plugin declares a
`Role` that says what it does, what it reads, and where it puts its output.

| Role      | Reads             | Writes to                  | On error            |
|-----------|--------------------|----------------------------|---------------------|
| Transform | `&mut Node`        | mutates the node in place  | n/a                 |
| Scan      | `&Node`            | `TransformMetadata` fields | n/a                 |
| Guard     | `&Node`, metadata  | nothing                    | `Err(CompileError)` via `finalize()` |

**Transform** rewrites AST nodes in place: renaming, desugaring, normalizing.
Runs first so every later plugin and the generator see the final form.

**Scan** reads the (already rewritten) AST and records facts into
`TransformMetadata` (e.g. "this template needs the `escape` helper," "this
template is async"). Never changes nodes.

**Guard** reads the AST or the metadata that Scan plugins wrote, and rejects
invalid programs by returning an error from `finalize()`. Never changes
anything. Runs last.

Ordering rule: Transform plugins run before any Scan or Guard that depends on
the rewritten forms. A Transform plugin never depends on metadata.

## The bare-minimum core (attributes)

The parser knows no sugar. For attributes it emits only:

- `Static { name, value }` for `name="literal"`.
- `Boolean { name }` for bare `name`.
- `Brace { name: Option<String>, raw, span }` for any `{...}` form, named
  (`x={raw}`) or anonymous (`{raw}`). The parser does not look inside `raw`.

Transform plugins turn `Brace` into the core forms the generator understands.
This is the Vue directiveTransforms model: pattern-keyed plugins, each owning
one feature.

| Transform plugin | Recognizes | Produces |
|---|---|---|
| SpreadLowering | anon `{**expr}` | `Spread { value_expr, value_span }` |
| SlotLowering | anon `{...name}` | `SlotAssignment { name, value_span }` |
| ShorthandLowering | anon `{name}` | `Dynamic { name, value_expr: name, value_span, mode }` |
| AttributeLowering | named `x={expr}`, quoted templates | `Dynamic { ... }` / `Template { ... }` |
| ReservedKeyword | `class` to `class_` | renames `value_expr` and kwarg keys |

`mode` is an `AttributeMode` (`Class`, `Style`, `Data`, `Aria`, `RenderAttribute`,
`Escape`) computed once, here, from the name and whether it was shorthand. The
generator switches on `mode` and never re-derives it. Helper detection reads the
same `mode`, so the rendering decision lives in one place.

Invariant: no `Brace` survives the plugin stage. The generator treats a leftover
`Brace` as a bug.

## Everything we do, bucketed

| Concern | Stage | Role | Lives today |
|---|---|---|---|
| tokenize | Lex | core | tokenizer.rs |
| build node tree | Parse | core | tree_builder.rs |
| unclosed tag, void-with-children | Parse | guard | tree_builder.rs |
| duplicate attributes | Parse | guard | tree_builder.rs |
| invalid nesting (block-in-p, interactive) | Parse | guard | tree_builder.rs |
| recognize `{**}` / `{...}` / `{name}` | Plugins | transform | tokenizer + tree_builder |
| attribute render mode | Plugins | transform | tree_builder + generator + helper_detect |
| `class` to `class_` | Plugins | transform | reserved_keyword plugin |
| helpers needed | Plugins | scan | helper_detect plugin |
| async detection | Plugins | scan | async_detect plugin |
| slots used | Plugins | scan | slot_detect plugin |
| implicit blessed spreads | Plugins | scan | spread_detect plugin |
| mutable default argument | Plugins | scan | mutable_default plugin |
| one blessed spread per template | Plugins | guard | spread_validate plugin |
| emit python plus raw ranges | Generate | core | python.rs, output.rs |
| brace spans, injection ranges | Map | core | brace_collector.rs, injection_analyzer.rs |
| drop ranges where source differs from compiled | Map | core | output.rs, inline in lib.rs |
| UTF-16, prefix/suffix injections | Map | core | output.rs |

## Source mapping

The most delicate subsystem. Spans must survive transformation, so a transformed
attribute keeps the original `{...}` `span`. Source mapping reads only spans,
never the attribute kind, so adding or changing Transform plugins cannot regress
it.

## Migration

Each stage lands green: all golden `.expected.*` unchanged except one documented fix.

1. **Home the floating validations.** ~~Label each existing plugin's role. Move the
   inline one-blessed-spread check from `lib.rs` into a Guard plugin. Leave the
   parse-time structural checks in Parse, where they belong. No new framework, no
   behavior change.~~ **Done.**

2. **Attribute core.** Add normalized `Dynamic { mode }` with uniform spans. A
   Transform plugin turns today's Expression and Shorthand into it. Remove the
   generator's name branching. The one expected output change: `data={x}` now
   imports `escape` instead of `render_data`, fixing a latent bug.

3. **Generic brace.** The lexer emits a generic brace. The parser emits surface
   `Brace`. SpreadLowering, SlotLowering, and ShorthandLowering produce the core
   forms. The parser stops knowing any sugar.
