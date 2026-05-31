# Roadmap

## v0.1.0

- [x] Rust transpiler (`.hyper` -> `.py`)
- [x] Component model with slots
- [x] All Python control flow
- [x] Streaming generator output
- [x] Attribute system (boolean, class, style, data, aria, spread)
- [x] Compile-time HTML validation
- [x] Content collections (JSON, YAML, TOML, Markdown)
- [x] JetBrains IDE plugin
- [x] TextMate syntax bundle
- [ ] PyPI package release
- [ ] JetBrains Marketplace release

## Next

- [ ] **Fix blank line handling** — `newline_is_content()` can't tell a line ending from a blank line. The generator works around this with `saturating_sub(1)`, which fixes component bodies but eats real blank lines (see `nested.hyper`).
- [ ] **Document whitespace semantics** in `docs/design/templates.md` and `docs/implementation/templates.md`
- [ ] Support multiline HTML tags (`<div\n  class="card">`)
- [ ] Interactive inspector: bidirectional source <-> compiled highlighting
- [ ] Collapse inspector to single tab with Python/HTML/Boilerplate toggles

## Known bugs

- [ ] **Blank lines before combinable content are swallowed** — a blank line between a comment/statement and an element disappears because `emit_combined_nodes` trims leading `\n`. Blank lines before statements work fine (not combinable). This is why the compiled output is missing blank lines between section headers and HTML content.
- [ ] **`yield from` invalid in async templates** — if a template uses `async for`/`async with`, the whole function becomes `async def`, making all `yield from Component()` calls invalid Python. The compiler needs to emit something different for async component calls.
- [ ] **Two blank lines between sections become one** — source has 2 blank lines, output has 1. Related to the combining/trimming issue above.
- [ ] **Newline between attributes breaks the tag** — when an opening tag splits attributes across lines (`<div id="x"\n class="y">`), the tokenizer closes the tag after the first attribute and emits the rest as text content. Only a single attribute's value may currently wrap (newlines inside the quotes are fine); workaround is to keep all attributes on the tag's first line with only the last value wrapping.
- [ ] **Parser panics on multibyte chars inside inline `{...}` expressions** — a non-ASCII char in an expression embedded in template text (e.g. `{x or "aprofundată"}`) panics at `tree_builder.rs:1172` with "byte index N is not a char boundary", because the span offset is computed in bytes not char boundaries. Workaround is to compute the string in the Python zone and emit an ASCII-named variable instead.
- [ ] **No way to emit a literal `{`/`}`** — any brace in template text or an attribute is read as an expression delimiter, which breaks inline JS/JSON (e.g. `onclick="if(c){f()}"` or `data-config='{"k":"v"}'`). `{{`/`}}` only stays literal in lines with no real expression (where it renders doubled, not collapsed), and `\{` produces broken Python; there is no escape that yields a single literal brace, so the only workaround is to avoid braces entirely or move the value into a `{expression}` that returns the string.
- [ ] **Spread `{**kwargs}` does not merge `class` with a literal `class`** — when a tag has both a literal `class="base"` and a `{**kwargs}` spread whose dict contains `class`, the output emits two separate `class` attributes (`class="base" class="caller"`) instead of merging them, so the browser silently drops the second. This makes the common "component with base classes + caller-supplied extra classes" pattern impossible via spread; the spread should detect `class` and concatenate it onto the literal one.

## Future

- [ ] File-based routing
- [ ] SSR framework integrations (FastAPI, Django, Flask)
- [ ] Static site generation
- [ ] Fragments for htmx partial rendering
- [ ] VS Code extension
