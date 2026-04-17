# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A component templating language for Python, written in Rust. Templates compile to typed Python functions with full IDE support.

```
uvx hyper .
```

## Quick Tour

### A simple template

A `.hyper` file compiles to a Python function.

```hyper
# Greeting.hyper

<h1>Hello, World!</h1>
```

```python
from components import Greeting

html = str(Greeting())
# <h1>Hello, World!</h1>
```

### Adding props

Props are type annotations above the `---` delimiter. They become keyword arguments.

```hyper
# Greeting.hyper

name: str
greeting: str = "Hello"

---

<h1>{greeting}, {name}!</h1>
```

```python
html = str(Greeting(name="World"))
# <h1>Hello, World!</h1>

html = str(Greeting(name="World", greeting="Hey"))
# <h1>Hey, World!</h1>
```

### Using Python in templates

The template body is the function body. Any valid Python works.

```hyper
items: list[str]

---

count = len(items)

<p>{count} items found</p>
<ul>
    for item in items:
        <li>{item}</li>
    end
</ul>
```

All Python control flow is supported: `if`/`elif`/`else`, `for`, `while`, `match`/`case`, `try`/`except`, `with`, and their `async` variants. Blocks end with `end` instead of relying on indentation.

### Composing components

Components compose like HTML elements. Use `<{Component}>` syntax to render one component inside another. Children go in the default slot with `{...}`.

```hyper
# Card.hyper
title: str

---

<div class="card">
    <h2>{title}</h2>
    <div class="card-body">
        {...}
    </div>
</div>
```

```hyper
# Page.hyper
from components import Card, Badge

items: list[dict]

---

for item in items:
    <{Card} title={item["name"]}>
        <{Badge} text="New" />
        <p>{item["description"]}</p>
    </{Card}>
end
```

### Streaming

Every component is a generator that yields HTML chunks. This means streaming works out of the box:

```python
from fastapi.responses import StreamingResponse
from pages import Feed

@app.get("/feed")
def feed():
    return StreamingResponse(Feed(posts=posts), media_type="text/html")
```

Or render to a string when you don't need streaming:

```python
html = str(Feed(posts=posts))
```

### Smart attributes

The compiler understands HTML attribute semantics, so you don't have to think about them.

```hyper
is_active: bool
disabled: bool

---

<!-- Boolean: True renders the attribute, False omits it -->
<button {disabled}>Click</button>

<!-- Class lists: strings, lists, and conditional dicts -->
<div class={["btn", {"active": is_active}]}>...</div>

<!-- Style objects -->
<p style={{"color": "red", "font-weight": "bold"}}>Alert</p>

<!-- Data/ARIA: dicts expand to prefixed attributes (ARIA bools become "true"/"false" per spec) -->
<div data={{"user-id": 123}} aria={{"label": "Close", "hidden": True}}>...</div>

<!-- Spread -->
<a {**attrs}>Link</a>
```

### Content collections

Load and validate structured content from Markdown, JSON, YAML, or TOML:

```python
from hyper.content import MarkdownCollection

class Post(MarkdownCollection):
    title: str
    date: str

    class Meta:
        pattern = "posts/*.md"

posts = Post.load()  # Typed, validated, with auto-generated html/toc/slug fields
```

Works with dataclasses, Pydantic, and msgspec.

## IDE Support

- **JetBrains** (PyCharm, IntelliJ) — Full Python intelligence inside `.hyper` files: autocomplete, go-to-definition, type checking, auto-transpilation on save
- **TextMate / VS Code** — Syntax highlighting

## Acknowledgements

Hyper's component and attribute syntax was inspired by [tdom](https://github.com/t-strings/tdom).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
