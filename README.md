# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A component templating language for Python, written in Rust.

```
uvx hyper .
```

### Why Hyper?

- Templates are real Python, not a restricted mini-language.
- Real components with slots and type-safe arguments.
- Full IDE support inside templates.

### Templates

**1. Write a template.** Props go above the `---`, template body below.

```hyper
# app/components/Button.hyper

label: str
disabled: bool = False
variant: str = "primary"

---

<button class={["btn", f"btn-{variant}"]} {disabled}>
    {label}
</button>
```

**2. Compile it.**

```
uvx hyper .
# ✓ app/components/Button.py
```

**3. Use it.**

```python
from app.components import Button

@app.get("/", response_class=HTMLResponse)
def index():
    return Button(label="Save", variant="danger")
```

### Control flow

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

### Components

Components compose like HTML elements. Children go in the default slot with `{...}`.

```hyper
# app/layouts/Layout.hyper

title: str = "My App"

---

<!doctype html>
<html>
<head>
    <title>{title}</title>
</head>
<body>
    {...}
</body>
</html>
```

```hyper
# app/pages/Dashboard.hyper
from app.layouts import Layout
from app.components import Card

items: list[dict]

---

<{Layout} title="Dashboard">
    for item in items:
        <{Card} title={item["name"]}>
            <p>{item["description"]}</p>
        </{Card}>
    end
</{Layout}>
```

Every component is a generator, so streaming works out of the box:

```python
from fastapi.responses import StreamingResponse
from app.pages import Dashboard

@app.get("/dashboard")
def dashboard():
    return StreamingResponse(Dashboard(items=items), media_type="text/html")
```

### IDE Support

- **JetBrains** (PyCharm, IntelliJ) — Full Python intelligence inside `.hyper` files: autocomplete, go-to-definition, type checking, auto-transpilation on save
- **TextMate / VS Code** — Syntax highlighting

### Acknowledgements

Hyper's component and attribute syntax was inspired by [tdom](https://github.com/t-strings/tdom).

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

### License

[MIT](LICENSE)
