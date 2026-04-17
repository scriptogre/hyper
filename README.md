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
# app/components/Greeting.hyper

name: str

---

<h1>Hello, {name}!</h1>
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

A more realistic component:

```hyper
# app/components/ProductCard.hyper

name: str
price: float
image: str
on_sale: bool = False

---

<div class={["card", {"sale": on_sale}]}>
    <img src={image} alt={name} />
    <h3>{name}</h3>
    if on_sale:
        <span class="badge">Sale</span>
    end
    <p class="price">${price:.2f}</p>
</div>
```

### Control flow

The template body is the function body. Any valid Python works. Blocks end with `end`.

```hyper
from app.enums import Status
from app.models import Product

status: Status
products: list[Product]

---

match status:
    case Status.LOADING:
        <div class="spinner" />
    case Status.ERROR:
        <p class="error">Something went wrong</p>
    case Status.OK:
        if not products:
            <p>No products found</p>
        else:
            <ul>
                for product in products:
                    <li>{product.name}</li>
                end
            </ul>
        end
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
