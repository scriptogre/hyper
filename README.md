# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/hyperhtml.svg)](https://pypi.org/project/hyperhtml/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Write HTML with Python. Import `.hyper` files without a build step or generated `.py` files.

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

The filename becomes the component name. Hyper compiles it on first import.

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

## Add a nested component

Use `component` for reusable markup inside the file:

```hyper
# app/pages/Dashboard.hyper
title: str
---
component Header(*, title: str):
    <header><h1>{title}</h1></header>
end

<{Header} title={title} />
```

The file exports `Dashboard` and its nested `Header`:

```python
from app.pages import Dashboard

Dashboard(title="Account")
Dashboard.Header(title="Account")
```

Both calls return:

```html
<header><h1>Account</h1></header>
```

`Dashboard("Account")` raises `TypeError` because props are keyword-only.

## Pass content

Place `{...}` where caller content belongs:

```hyper
# app/components/Card.hyper
title: str
---
<article>
    <h2>{title}</h2>
    <main>{...}</main>
</article>
```

Pass content between component tags:

```hyper
# app/pages/Confirm.hyper
from app.components import Card
---
<{Card} title="Delete item">
    <p>This cannot be undone.</p>
</{Card}>
```

```html
<article><h2>Delete item</h2><main><p>This cannot be undone.</p></main></article>
```

## Group components in one file

A file with declarations and no rendered output is a component library:

```hyper
# app/components/forms.hyper
component Button(*, label: str, **attrs):
    <button {**attrs}>{label}</button>
end

component Input(*, name: str, type: str = "text"):
    <input {name} {type} />
end
```

Import its components like a Python module:

```python
from app.components.forms import Button, Input

Button(label="Save", disabled=True)
Input(name="email", type="email")
```

```html
<button disabled>Save</button>
<input name="email" type="email">
```

## Use Python

Python statements and expressions stay Python:

```hyper
items: list[str]
show_count: bool = True
---
<ul>
    for item in items:
        <li>{item.upper()}</li>
    end
</ul>

if show_count:
    <p>{len(items)} items</p>
end
```

Imports, functions, classes, `match`, `try`, `with`, and async code use the same block form.

## Split long tags

Formatting whitespace between attributes does not render:

```hyper
<{Button}
    label="Save"
    class={["button", {"active": active}]}
    hx-post="/save"
/>
```

## Stream output

Calling a component joins its chunks. `.stream()` returns the generated iterator:

```python
html = Feed(posts=posts)
chunks = Feed.stream(posts=posts)
```

```python
from fastapi.responses import StreamingResponse


@app.get("/feed")
def feed():
    return StreamingResponse(
        Feed.stream(posts=posts),
        media_type="text/html",
    )
```

## Use Django or Jinja

Django views can return a component directly:

```python
from django.http import HttpResponse


def home(request):
    return HttpResponse(Dashboard(title="Account"))
```

Jinja can discover `.hyper` files from its template paths:

```python
env.add_extension("hyperhtml.integrations.jinja2.HyperExtension")
```

```jinja
{{ Dashboard(title="Account") }}
```

See [Integrations](docs/design/integrations.md) for Django templates, Jinja slots, Flask, Litestar, and Sanic.

## How files import

| `.hyper` file | Python import |
| --- | --- |
| Contains rendered output or `---` | Component named after the file |
| Contains declarations only | Normal module with exported names |
| Has an adjacent `.py` file | Python file wins |

Implicit file components must live inside a package. Root-level `.hyper` files can be component libraries.

## IDE support

- **JetBrains:** syntax highlighting, Python language injection, and compiler diagnostics
- **TextMate and VS Code:** syntax highlighting

## Documentation

- [Template language](docs/design/templates.md)
- [Integrations](docs/design/integrations.md)
- [Contributing](CONTRIBUTING.md)

## License

[MIT](LICENSE)
