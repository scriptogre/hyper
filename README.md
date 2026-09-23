# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/hyperhtml.svg)](https://pypi.org/project/hyperhtml/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Write type-safe HTML in `.hyper` files. Import components directly from Python.

```bash
uv add hyperhtml
```

Requires Python 3.10 or newer.

## Create a component

Create `app/pages/Greeting.hyper`:

```html
<h1>Hello, World!</h1>
```

Import and call it:

```python
from app.pages import Greeting

print(Greeting())
```

```html
<h1>Hello, World!</h1>
```

The filename sets the component name.

## Pass data

Add typed props above `---`:

```html
name: str
---
<h1>Hello, {name}!</h1>
```

```python
print(Greeting(name="Ada"))
```

```html
<h1>Hello, Ada!</h1>
```

Props are keyword-only. Values are escaped by default.

Calling a component binds its props. Rendering starts when it becomes a string or when you call `.render()`:

```python
greeting = Greeting(name="Ada")
html = greeting.render()
```

## Use Python

Use Python expressions and control flow directly:

```html
names: list[str]
---
for name in names:
    <h1>Hello, {name}!</h1>
```

```python
print(Greeting(names=["Ada", "Lin"]))
```

```html
<h1>Hello, Ada!</h1><h1>Hello, Lin!</h1>
```

Indentation defines Python blocks. Add an aligned `end` only when it makes a boundary clearer.

## Compose components

Create `app/components/Card.hyper`:

```html
title: str
---
<article><h2>{title}</h2></article>
```

Use it from `app/pages/Dashboard.hyper`:

```html
from app.components import Card
---
<{Card} title="Orders" />
```

```python
from app.pages import Dashboard

print(Dashboard())
```

```html
<article><h2>Orders</h2></article>
```

## Pass content

Place `{...}` where caller content belongs:

```html
title: str
---
<article>
    <h2>{title}</h2>
    <div>{...}</div>
</article>
```

Pass content between component tags:

```html
from app.components import Card
---
<{Card} title="Orders">
    <p>3 open orders</p>
</{Card}>
```

```html
<article><h2>Orders</h2><div><p>3 open orders</p></div></article>
```

## Pass named content

Use a named slot when content belongs in a specific place:

```html
title: str
---
<article>
    <h2>{title}</h2>
    <div>{...}</div>
    <footer>{...actions}</footer>
</article>
```

Mark the element that fills it:

```html
from app.components import Card
---
<{Card} title="Orders">
    <p>3 open orders</p>
    <button {...actions}>View orders</button>
</{Card}>
```

```html
<article><h2>Orders</h2><div><p>3 open orders</p></div><footer><button>View orders</button></footer></article>
```

## Define several components

Use `def ... -> Component` to group related components in one file:

```html
# app/components/forms.hyper
from hyper import Component

def Button(*, label: str) -> Component:
    <button>{label}</button>

def Input(*, name: str) -> Component:
    <input name={name} />
```

A declarations-only file imports like a Python module:

```python
from app.components.forms import Button, Input

print(Button(label="Save"))
```

```html
<button>Save</button>
```

Import each name used by the source:

```python
from hyper import Component
```

## Use FastAPI

Set the response type to HTML, then render a component:

```python
from fastapi import FastAPI
from fastapi.responses import HTMLResponse

from app.pages import Dashboard

app = FastAPI(default_response_class=HTMLResponse)


@app.get("/dashboard")
def dashboard():
    return Dashboard().render()
```

## Stream a response

Pass `stream=True` to send each generated chunk:

```python
from fastapi.responses import StreamingResponse


@app.get("/dashboard/stream")
def stream_dashboard():
    return StreamingResponse(
        Dashboard().render(stream=True),
        media_type="text/html",
    )
```

See [Integrations](docs/design/integrations.md) for Django, Jinja, Flask, Litestar, and Sanic.

## IDE support

- **JetBrains:** syntax highlighting, Python language injection, and compiler diagnostics
- **TextMate and VS Code:** syntax highlighting

## Documentation

- [Template language](docs/design/templates.md)
- [Integrations](docs/design/integrations.md)
- [Contributing](CONTRIBUTING.md)

## License

[MIT](LICENSE)
