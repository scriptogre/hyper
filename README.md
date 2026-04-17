# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A Python framework for hypermedia-driven applications. Write templates in `.hyper` syntax that compile to type-safe, streaming Python.

## Usage

Write a template:

```hyper
# Button.hyper
label: str
variant: str = "primary"
disabled: bool = False

---

<button class="btn btn-{variant}" {disabled}>
    {label}
</button>
```

Use it:

```python
from Button import Button

html = str(Button(label="Save", variant="danger"))
# <button class="btn btn-danger">Save</button>
```

## Features

- **Type-safe props** — Catch errors before runtime with Python type annotations
- **Streaming by default** — Components are generators, ready for HTTP streaming
- **Full Python** — Any expression, import, or control flow inside templates
- **IDE intelligence** — Autocomplete, go-to-definition, and type checking in JetBrains
- **Rust-powered compiler** — Fast compilation with helpful error messages
- **Content collections** — Load and validate Markdown, JSON, YAML, TOML with typed models

## Components and Slots

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
    <{Card} title={item['name']}>
        <{Badge} text="New" />
        <p>{item['description']}</p>
    </{Card}>
end
```

## Control Flow

All Python control flow works inside templates — `if`/`elif`/`else`, `for`, `while`, `match`/`case`, `try`/`except`, `with`, and `async` variants. Blocks end with `end` instead of relying on indentation.

```hyper
status: str

---

match status:
    case "loading":
        <div class="spinner" />
    case "error":
        <p class="error">Something went wrong</p>
    case _:
        <p>Ready</p>
end
```

## Streaming

Components are generators that yield chunks, ready for HTTP streaming out of the box:

```python
from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from pages import Feed

app = FastAPI()

@app.get("/feed")
def feed():
    return StreamingResponse(Feed(posts=posts), media_type="text/html")
```

Or render to a string:

```python
html = str(Feed(posts=posts))
```

## Attributes

```hyper
is_active: bool
data: dict = {"user-id": 123}

---

# Boolean — renders as <button disabled> or <button>
<button {disabled}>Click</button>

# Class — lists, dicts, conditional
<div class={["btn", {"active": is_active}]}>...</div>

# Style — dict to inline CSS
<p style={{"color": "red"}}>Alert</p>

# Data/ARIA — auto-prefixed
<div {data} aria={{"label": "Close"}}>...</div>

# Spread
<a {**attrs}>Link</a>
```

## Content Collections

Load and validate structured content from JSON, YAML, TOML, or Markdown files:

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

## Editor Support

- **JetBrains** (PyCharm, IntelliJ) — Full Python intelligence inside `.hyper` files: autocomplete, go-to-definition, type checking, auto-transpilation on save
- **TextMate** — Syntax highlighting bundle

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
