# Hyper runtime

Runtime helpers, integrations, and import hook for `.hyper` templates.

## Installation

```bash
uv add hyperhtml
```

Requires Python 3.10+.

## Quick start

Write a component in `.hyper`:

```hyper
# Greeting.hyper
name: str
---
<h1>Hello {name}</h1>
```

Import it like a Python component:

```python
from Greeting import Greeting

Greeting(name="Ada")        # HtmlResult('<h1>Hello Ada</h1>')
```

The import hook compiles `.hyper` files in memory. It does not write `.py` files.

A compiled component is a plain `@html`-decorated callable. It escapes its own
arguments and marks its output safe under MarkupSafe.

## Integrations

Components are just callables returning strings, so they fit any engine. See
[docs/design/integrations.md](../docs/design/integrations.md) for the full guide
(slots, named slots, `**spread`).

**Jinja2.** Add the extension; components in the loader's paths become globals:

```python
env.add_extension("hyperhtml.integrations.jinja2.HyperExtension")
# {{ Greeting(name="Ada") }}
# {% hyper Card(title="Pricing") %}…{% slot actions %}…{% endslot %}{% endhyper %}
```

**Django.** Add the app, register the tag as a builtin:

```python
INSTALLED_APPS = ["hyperhtml.integrations.django", ...]
# {% hyper Greeting name=user.first_name / %}
# {% hyper Card title="Pricing" %}…{% slot actions %}…{% endslot %}{% endhyper %}
```

**FastAPI / Flask.** No integration needed, return the component:

```python
return HTMLResponse(Greeting(name="Ada"))
return StreamingResponse(Greeting.stream(name="Ada"))   # chunk-by-chunk
```

## Runtime helpers

Compiled templates import what they use from `hyperhtml`. You rarely call these
directly, but they make up the generated output:

### Escaping

| Function | Purpose |
|----------|---------|
| `escape(value)` | Escape HTML special characters |
| `safe(value)` | Mark content as safe (no escaping) |

```python
escape("<script>")    # "&lt;script&gt;"
safe("<b>bold</b>")   # "<b>bold</b>"
```

### Attributes

| Function | Purpose |
|----------|---------|
| `render_attr(name, value)` | Render a single attribute |
| `render_class(*values)` | Render a class attribute |
| `render_style(value)` | Render a style attribute |
| `spread_attrs(attrs)` | Spread a dict as attributes (`{**attrs}`) |

```python
render_attr("disabled", True)        # " disabled"
render_attr("disabled", False)       # ""
render_attr("id", "main")            # ' id="main"'

render_class("btn", {"active": True})  # "btn active"
render_style({"color": "red"})         # "color:red"
spread_attrs({"class": "btn"})         # ' class="btn"'
```
