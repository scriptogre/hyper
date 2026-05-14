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

- [ ] **Newline semantics in tree builder** — `newline_is_content()` treats all newlines after content tokens as content, but the first newline after an opening tag/statement is a structural line ending, not a blank line. Fix at the tree builder level so the generator doesn't need `saturating_sub(1)` workarounds.
- [ ] Support multiline HTML tags (`<div\n  class="card">`)
- [ ] Interactive inspector: bidirectional source <-> compiled highlighting
- [ ] Collapse inspector to single tab with Python/HTML/Boilerplate toggles

## Future

- [ ] File-based routing
- [ ] SSR framework integrations (FastAPI, Django, Flask)
- [ ] Static site generation
- [ ] Fragments for htmx partial rendering
- [ ] VS Code extension
