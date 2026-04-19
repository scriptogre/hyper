# TODO

## Compiler bugs

- **Blank lines before combinable content are swallowed** — a blank line between a comment/statement and an element disappears because `emit_combined_nodes` trims leading `\n`. Blank lines before statements work fine (not combinable). This is why the compiled output is missing blank lines between section headers and HTML content.

- **`yield from` invalid in async templates** — if a template uses `async for`/`async with`, the whole function becomes `async def`, making all `yield from Component()` calls invalid Python. The compiler needs to emit something different for async component calls.

- **Two blank lines between sections become one** — source has 2 blank lines, output has 1. Related to the combining/trimming issue above.
