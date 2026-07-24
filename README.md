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

```hyper
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

```hyper
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

## Use Python

Use Python expressions and control flow directly:

```hyper
names: list[str]
---
for name in names:
    <h1>Hello, {name}!</h1>
end
```

```python
print(Greeting(names=["Ada", "Lin"]))
```

```html
<h1>Hello, Ada!</h1><h1>Hello, Lin!</h1>
```

Close each indented block with `end`.

## Compose components

Create `app/components/Card.hyper`:

```hyper
title: str
---
<article><h2>{title}</h2></article>
```

Use it from `app/pages/Dashboard.hyper`:

```hyper
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

```hyper
title: str
---
<article>
    <h2>{title}</h2>
    <div>{...}</div>
</article>
```

Pass content between component tags:

```hyper
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

```hyper
title: str
---
<article>
    <h2>{title}</h2>
    <div>{...}</div>
    <footer>{...actions}</footer>
</article>
```

Mark the element that fills it:

```hyper
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

Use `component` to group related components in one file:

```hyper
# app/components/forms.hyper
component Button(*, label: str):
    <button>{label}</button>
end

component Input(*, name: str):
    <input name={name} />
end
```

A declarations-only file imports like a Python module:

```python
from app.components.forms import Button, Input

print(Button(label="Save"))
```

```html
<button>Save</button>
```

## Use FastAPI

Set the response type to HTML, then return a component:

```python
from fastapi import FastAPI
from fastapi.responses import HTMLResponse

from app.pages import Dashboard

app = FastAPI(default_response_class=HTMLResponse)


@app.get("/dashboard")
def dashboard():
    return Dashboard()
```

## Stream a response

Use `.stream()` to send each generated chunk:

```python
from fastapi.responses import StreamingResponse


@app.get("/dashboard/stream")
def stream_dashboard():
    return StreamingResponse(
        Dashboard.stream(),
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
