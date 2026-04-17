# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A component templating language for Python, written in Rust.

```
uvx hyper .
```

`.hyper` files compile to real Python with type-safe components, slots, and full IDE support.

### Quick Start

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

### Features

#### Components

Components compose like HTML elements. Children go in the default slot with `{...}`.

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

```hyper
# app/pages/Store.hyper
from app.layouts import Layout
from app.components import ProductCard

products: list[Product]

---

<{Layout} title="Store">
    for product in products:
        <{ProductCard}
            name={product.name}
            price={product.price}
            image={product.image}
            on_sale={product.on_sale}
        />
    end
</{Layout}>
```

#### Control flow

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

#### Streaming

Every component is a generator, so streaming works out of the box:

```python
from fastapi.responses import StreamingResponse
from app.pages import Store

@app.get("/store")
def store():
    return StreamingResponse(Store(products=products), media_type="text/html")
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
