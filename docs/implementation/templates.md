# Templates

`.hyper` files compile to Python generator functions that yield HTML chunks.

## Basic Example

Write this:

```hyper
name: str
count: int = 0

---

<div>
    <h1>Hello {name}</h1>
    if count > 0:
        <p>You have {count} items</p>
    end
</div>
```

Get this:

```python
from hyper import html, replace_markers


@html
def Template(*, name: str, count: int = 0):
    yield replace_markers(f"""\
<div>
    <h1>Hello ‹ESCAPE:{name}›</h1>""")

    if count > 0:
        yield replace_markers(f"""\
<p>You have ‹ESCAPE:{count}› items</p>
    """)

    yield "</div>"
```

The `@html` decorator handles string conversion — `str(Template(name="World"))` joins all yielded chunks. Iterating directly enables streaming.

Transpile once. Execute many times.

---

## The `---` Delimiter

Each `.hyper` file is a Python module. The `---` separates module-level code from the function body.

### Above `---`: Module-Level

Type hints become keyword-only function parameters:

```hyper
title: str
count: int = 0
**kwargs: dict
---
<div>{title}</div>
```

```python
from hyper import html, replace_markers


@html
def Template(*, title: str, count: int = 0, **kwargs: dict):
    yield replace_markers(f"""<div>‹ESCAPE:{title}›</div>""")
```

### Below `---`: Function Body

**This is regular Python.** Write any code you'd write in a function body: variables, loops, functions, classes, control flow.

The magic: Write HTML directly. Use `{expr}` for interpolation.

```hyper
---
items = ["Apple", "Banana", "Cherry"]

<ul>
    for item in items:
        <li>{item}</li>
    end
</ul>
```

```python
@html
def Template():

    items = ["Apple", "Banana", "Cherry"]

    yield "<ul>"

    for item in items:
        yield replace_markers(f"""\
<li>‹ESCAPE:{item}›</li>
    """)

    yield "</ul>"
```

### The `end` Keyword

`end` is required for control flow inside HTML-rendering contexts — below `---`, or inside `def` blocks that emit HTML:

```hyper
---
for item in items:
    <li>{item}</li>
end

if is_active:
    <span>Active</span>
end
```

Without `end`, the parser can't tell where blocks end (HTML closing tags don't signal block boundaries to the parser).

**Above `---`**, blocks scope by indentation like normal Python. No `end` needed:

```hyper
def format_date(date):
    return date.strftime("%Y-%m-%d")

def Card(title: str):
    <div class="card">
        <h2>{title}</h2>
        if show:
            <p>Details</p>
        end
    </div>

name: str
---
```

Both `def` blocks end via dedentation. The `if` inside `Card` needs `end` because it's inside an HTML-rendering context.

### Add Imports

Imports go above `---`:

```hyper
from components import Button

title: str
---
<{Button} label={title} />
```

```python
from components import Button
from hyper import html


@html
def Template(*, title: str):
    yield from Button(label=title)
```

### Add Helpers

Functions and classes go above `---`:

```hyper
def format_date(date):
    return date.strftime("%Y-%m-%d")

created_at: datetime
---
<p>Created: {format_date(created_at)}</p>
```

```python
def format_date(date):
    return date.strftime("%Y-%m-%d")


@html
def Template(*, created_at: datetime):
    yield replace_markers(f"""<p>Created: ‹ESCAPE:{format_date(created_at)}›</p>""")
```

Pure Python helpers above `---` use normal Python scoping — no `end` needed.

Functions and classes are importable:

```python
from components.utils import format_date
```

### Add Constants

Use `Final` annotation. Not a parameter.

```hyper
from typing import Final

MAX_ITEMS: Final[int] = 100

items: list
---
<p>Showing {len(items[:MAX_ITEMS])} items</p>
```

```python
from typing import Final
from hyper import html, replace_markers

MAX_ITEMS: Final[int] = 100


@html
def Template(*, items: list):
    yield replace_markers(f"""<p>Showing ‹ESCAPE:{len(items[:MAX_ITEMS])}› items</p>""")
```

Constants stay at module level.

### When `---` Is Optional

**Only HTML:**

```hyper
<div>Hello</div>
```

No `---` needed. Everything is template body.

**Only definitions:**

```hyper
def helper():
    return "value"

class Component:
    pass
```

No `---` needed. Everything is module-level. No render function generated. This is a library file.

**Mixed content needs `---`:**

```hyper
def helper():
    return "value"
---
<div>{helper()}</div>
```

---

## How Transpilation Works

Three stages:

```
Source → Parser → Transformer → Generator → Python
```

### Parser

Splits on `---`. Builds AST. Tracks positions for errors and IDE.

### Transformer

Collects info: which helpers are used, async detection, slot parameters.

### Generator: Yield-Based Streaming

The generator emits `yield` statements instead of appending to a list. Static HTML yields plain strings. Dynamic content yields f-strings processed by `replace_markers`.

Special attributes use markers:

```hyper
class = ["btn", "active"]
<div {class}>
```

```python
_class = ["btn", "active"]
yield replace_markers(f"""<div class=‹CLASS:{_class}›>""")
```

**Why markers?** IDE sees `_class`, not `render_class(_class)`. Autocomplete works. `replace_markers` processes them at runtime.

---

## CLI Usage

```bash
# Single file
hyper generate Button.hyper

# Directory
hyper generate components/

# Stdin (IDE integration)
hyper generate --stdin --json --injection
```

File structure:

```
components/
├── Button.hyper    # Source
├── Button.py       # Generated (gitignored)
└── __init__.py     # Auto-generated exports
```

---

## Using Templates

Import like normal Python:

```python
from components import Button

html = str(Button(label="Click me", disabled=False))
```

Templates return generators. Use `str()` for buffered output, or iterate for streaming.

### Framework Integration

Works with any framework.

**FastAPI (streaming):**
```python
from fastapi.responses import StreamingResponse
from pages import Home

@app.get("/")
def index():
    return StreamingResponse(
        Home(title="Welcome"),
        media_type="text/html"
    )
```

**FastAPI (buffered):**
```python
from fastapi.responses import HTMLResponse
from pages import Home

@app.get("/")
def index():
    return HTMLResponse(str(Home(title="Welcome")))
```

**Django / Flask:** Same patterns. Import template, call function, iterate or `str()`.

---

## Generated Code Patterns

Type hints → keyword-only parameters:

```hyper
title: str
is_active: bool = False
```
```python
@html
def Template(*, title: str, is_active: bool = False):
```

Control flow → Python with yields:

```hyper
if active:
    <span>Active</span>
end
```
```python
if active:
    yield """<span>Active</span>"""
```

Loops → Python with yields:

```hyper
for item in items:
    <li>{item.name}</li>
end
```
```python
for item in items:
    yield replace_markers(f"""<li>‹ESCAPE:{item.name}›</li>""")
```

Components → yield from:

```hyper
<{Button} label="Save" />
```
```python
yield from Button(label="Save")
```

---

## IDE Integration

Transpiler tracks source positions to compiled positions:

```json
{
  "ranges": [{
    "type": "python",
    "source_start": 21,
    "source_end": 27,
    "compiled_start": 94,
    "compiled_end": 108
  }]
}
```

IDE splices source into compiled context. Python tooling works on `.hyper` files.

CLI outputs JSON for IDE plugins:

```bash
hyper generate --stdin --json --injection
```

---

## Security

### Auto-Escaping

All expressions are escaped via markers:

```hyper
<div>{user_input}</div>
```
```python
yield replace_markers(f"""<div>‹ESCAPE:{user_input}›</div>""")
```

`replace_markers` escapes values at runtime. Static HTML (no expressions) yields plain strings without processing.

### Raw HTML

Use `safe()` for trusted content:

```hyper
<div>{safe(html_content)}</div>
```

Only use `safe()` on content you control.

### Static Code Generation

No `eval()`. No runtime execution. What you transpile is what runs.

Control imports via:
- Virtual environments
- Import hooks (for sandboxing)
- Code review

---

## Compile-Time Validation

Validates templates before code runs.

**Implemented:**
- Unclosed tags
- Mismatched tags
- Void element children
- Duplicate attributes
- Invalid HTML nesting (block in inline, nested interactive)

**Planned:**
- Type mismatches in component props
- Unknown props
- Missing required props
- Missing accessibility attributes
- Slot validation

---

## Summary

- `.hyper` → `.py` at build time
- Yield-based generators enable streaming
- `@html` decorator handles `str()` conversion
- Auto-escaping via markers prevents XSS
- Source maps for IDE support
- `end` keyword required in HTML-rendering contexts, not for pure Python above `---`

---

**See Also:**
- [Templates Syntax](../design/templates.md) - Language guide
