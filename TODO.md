# IDE Injection TODO

Remaining work for proper syntax highlighting in .hyper files.

## Current Status

Working:
- [x] Frontmatter parameters (`name: str`) - Python highlighting
- [x] Expressions with braces (`{name}`, `{count + 1}`) - f-string highlighting with orange braces
- [x] Control flow conditions (`is_active` in `if is_active:`) - partial

## Remaining Work

### 1. HTML Injection
HTML elements are not injected at all. Need to add `RangeType::Html` ranges for:
- Opening tags: `<div class="container">`
- Closing tags: `</div>`
- Self-closing tags: `<br/>`
- Full elements when they contain no expressions: `<span>Plain text</span>`

### 2. Comments
Comments (`# comment`) are not injected. Should be Python so `#` gets comment highlighting.
- Frontmatter comments
- Body comments (standalone lines)
- Trailing comments (after HTML content)

### 3. Full Control Flow Statements
Currently only conditions are injected. Should inject full statements:
- `if condition:` (not just `condition`)
- `elif condition:`
- `else:`
- `for item in items:`
- `while condition:`
- `match expr:`
- `case pattern:`
- `with expr:`
- `end` keywords

### 4. Comprehensive Test Coverage
The test file at `rust/transpiler/tests/injections/comprehensive.hyper` defines expected injections.
Update snapshots once all injections are implemented correctly.

## Design Notes

The injection system maps source positions to a "virtual Python file" using prefix/suffix:
- `prefix` = compiled code before the source range
- `suffix` = compiled code after the source range
- IDE sees: `prefix + source_text + suffix`

For HTML injection, we may need a separate virtual HTML file or use nested injection.
