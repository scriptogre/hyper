# Hyper

Runtime helpers and CLI for Hyper HTML templates.

## Installation

```bash
pip install hyper
```

Requires Python 3.10+.

## CLI

Compile `.hyper` templates to Python:

```bash
# Single file
hyper generate Button.hyper

# Directory (generates __init__.py with exports)
hyper generate components/

# From stdin
echo '<div>{message}</div>' | hyper generate --stdin
```

## Runtime Helpers

Compiled templates import these helpers:

```python
from hyper import _e, safe, _attr, _class, _style, _spread
```

### Escaping

| Function | Purpose |
|----------|---------|
| `_e(value)` | Escape HTML special characters |
| `safe(value)` | Mark content as safe (no escaping) |

```python
_e("<script>")      # "&lt;script&gt;"
safe("<b>bold</b>") # "<b>bold</b>"
```

### Attributes

| Function | Purpose |
|----------|---------|
| `_attr(name, value)` | Render a single attribute |
| `_class(*values)` | Render class attribute |
| `_style(value)` | Render style attribute |
| `_spread(attrs)` | Spread dict as attributes |

```python
_attr("disabled", True)   # " disabled"
_attr("disabled", False)  # ""
_attr("id", "main")       # ' id="main"'

_class("btn", {"active": True})  # "btn active"
_style({"color": "red"})         # "color:red"
_spread({"class": "btn"})        # ' class="btn"'
```
