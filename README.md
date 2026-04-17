# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A component templating language for Python, written in Rust.

```
uvx hyper .
```

## Why Hyper?

- Templates are real Python, not a restricted mini-language.
- Real components with slots and type-safe arguments.
- Full IDE support inside templates.

## Quick Tour

### Templates

**1. Write a template.** Props go above the `---`, template body below.

```hyper
# app/components/Greeting.hyper

name: str                    # required
greeting: str = "Hello"      # optional

---

<h1>{greeting}, {name}!</h1>
```

**2. Compile it.**

```
uvx hyper .
# ✓ app/components/Greeting.py
```

**3. Use it.**

```python
from app.components import Greeting

@app.get("/", response_class=HTMLResponse)
def index():
    return Greeting(name="World")
```

### Use Python in templates

The template body is the function body. Any valid Python works. Blocks end with `end`.

```hyper
status: str
items: list[dict]

---

if not items:
    <p>No items found</p>
elif status == "loading":
    <div class="spinner" />
else:
    <ul>
        for item in items:
            <li>{item["name"]}</li>
        end
    </ul>
end

match status:
    case "error":
        <p class="error">Something went wrong</p>
    case _:
        <p>Items: {len(items)}</p>
end
```

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
from app.components import Card, Badge

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
from app.pages import Feed

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

## IDE Support

- **JetBrains** (PyCharm, IntelliJ) — Full Python intelligence inside `.hyper` files: autocomplete, go-to-definition, type checking, auto-transpilation on save
- **TextMate / VS Code** — Syntax highlighting

## Acknowledgements

Hyper's component and attribute syntax was inspired by [tdom](https://github.com/t-strings/tdom).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
