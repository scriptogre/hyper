# Templates

`.hyper` files compile to Python functions that return HTML strings.

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
from hyper import escape

def Template(name: str, count: int = 0) -> str:
    _parts = []
    _parts.append(f"""<div>
    <h1>Hello {escape(name)}</h1>
    """)
    if count > 0:
        _parts.append(f"""<p>You have {escape(count)} items</p>
    """)
    _parts.append(f"""</div>""")
    return ''.join(_parts)
```

Transpile once. Execute many times.

---

## The `---` Delimiter

Each `.hyper` file is a Python module. The `---` separates module-level code from the function body.

### Above `---`: Module-Level

Type hints become function parameters:

```hyper
title: str
count: int = 0
**kwargs: dict
---
<div>{title}</div>
```

```python
def Template(title: str, count: int = 0, **kwargs: dict) -> str:
    _parts = []
    _parts.append(f"""<div>{escape(title)}</div>""")
    return ''.join(_parts)
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
def Template() -> str:
    _parts = []
    items = ["Apple", "Banana", "Cherry"]

    _parts.append("""<ul>""")
    for item in items:
        _parts.append(f"""<li>{escape(item)}</li>""")
    _parts.append("""</ul>""")
    return ''.join(_parts)
```

**The `end` keyword:** Required for control flow and definitions. Matches HTML's visual block structure.

```hyper
---
for item in items:
    <li>{item}</li>
end  # Required

if is_active:
    <span>Active</span>
end  # Required
```

Without `end`, the parser can't tell where blocks end (HTML has closing tags, Python doesn't).

**Define functions below `---`:**

```hyper
---
def calculate_total(items):
    return sum(item.price for item in items)
end

total = calculate_total(cart_items)
<p>Total: ${total}</p>
```

Any Python. Plus HTML.

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

def Template(title: str) -> str:
    _parts = []
    _parts.append(Button(label=title))
    return ''.join(_parts)
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

def Template(created_at: datetime) -> str:
    _parts = []
    _parts.append(f"""<p>Created: {escape(format_date(created_at))}</p>""")
    return ''.join(_parts)
```

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

MAX_ITEMS: Final[int] = 100

def Template(items: list) -> str:
    _parts = []
    _parts.append(f"""<p>Showing {escape(len(items[:MAX_ITEMS]))} items</p>""")
    return ''.join(_parts)
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

### Generator: Marker-Based Rendering

Special attributes use markers:

```hyper
class = ["btn", "active"]
<div {class}>
```

```python
_class = ["btn", "active"]
_parts.append(f"""<div class=‹CLASS:{_class}›>""")
return replace_markers("".join(_parts))
```

**Why?** IDE sees `_class`, not `render_class(_class)`. Autocomplete works. Runtime processes marker.

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

html = Button(label="Click me", disabled=False)
```

### Framework Integration

Works with any framework. Templates return strings.

**FastAPI:**
```python
from fastapi.responses import HTMLResponse
from pages import Home

@app.get("/")
def index():
    return HTMLResponse(Home(title="Welcome"))
```

**Django / Flask:** Same pattern. Import template, call function, return response.

---

## Generated Code Patterns

Type hints → parameters:

```hyper
title: str
is_active: bool = False
```
```python
def Template(title: str, is_active: bool = False) -> str: ...
```

Control flow → Python:

```hyper
if active:
    <span>Active</span>
end
```
```python
if active:
    _parts.append(f"""<span>Active</span>""")
```

Loops → Python:

```hyper
for item in items:
    <li>{item.name}</li>
end
```
```python
for item in items:
    _parts.append(f"""<li>{escape(item.name)}</li>""")
```

Components → function calls:

```hyper
<{Button} label="Save" />
```
```python
_parts.append(Button(label="Save"))
```

---

## IDE Integration

> **Status**: 🚧 Actively being refined

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

All expressions escape HTML:

```hyper
<div>{user_input}</div>
```
```python
_parts.append(f"""<div>{escape(user_input)}</div>""")
```

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

> **Status**: 🔮 Planned

Validates templates before code runs.

**Implemented:**
- Unclosed tags
- Mismatched tags

**Planned:**
- Type mismatches in component props
- Unknown props
- Missing required props
- Invalid HTML nesting
- Missing accessibility attributes
- Invalid boolean attributes
- Slot validation

Configure strictness:

```toml
[validation]
strict = true  # Warnings become errors
```

---

## Streaming Responses

> **Status**: 🔮 Exploring

Potential async iteration for large responses:

```python
from fastapi.responses import StreamingResponse

@app.get("/feed")
def feed():
    return StreamingResponse(
        Feed(posts=all_posts),
        media_type="text/html"
    )
```

Template stays the same. Framework detects async iteration.

Design TBD.

---

## Summary

- `.hyper` → `.py` at build time
- Framework-agnostic functions
- Auto-escaping prevents XSS
- Source maps for IDE support (evolving)
- Validation planned

---

**See Also:**
- [Templates Syntax](../design/templates.md) - Language guide