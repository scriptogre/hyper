# Hyper

[![CI](https://github.com/scriptogre/hyper/actions/workflows/ci.yml/badge.svg)](https://github.com/scriptogre/hyper/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/hyperhtml.svg)](https://pypi.org/project/hyperhtml/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Write type-safe HTML in `.hyper` files and import it directly from Python. Hyper compiles templates in memory, with no CLI, build step, or generated `.py` files.

```bash
uv add hyperhtml
```

Requires Python 3.10 or newer.

## Render a component

Create `app/templates/Greeting.hyper`:

```hyper
name: str
---
<h1>Hello, {name}!</h1>
```

Import it in a FastAPI endpoint:

```python
from fastapi import FastAPI
from fastapi.responses import HTMLResponse

from app.templates import Greeting

app = FastAPI()

@app.get("/", response_class=HTMLResponse)
def home():
    return Greeting(name="World")
```

The response is:

```html
<h1>Hello, World!</h1>
```

Hyper compiles `Greeting.hyper` on first import and caches it for the life of the process.

## Use Django

Return a component directly from a Django view:

```python
from django.http import HttpResponse

from app.templates import Greeting


def home(request):
    return HttpResponse(Greeting(name="World"))
```

To call Hyper components from Django templates, install the extra:

```bash
uv add "hyperhtml[django]"
```

Add the app, context processor, and builtin to your existing Django settings:

```python
INSTALLED_APPS = [
    # ...
    "hyperhtml.integrations.django",
]

TEMPLATES = [{
    # ...
    "OPTIONS": {
        "context_processors": [
            # ...
            "hyperhtml.integrations.django.context_processors.components",
        ],
        "builtins": [
            "hyperhtml.integrations.django.templatetags.hyper",
        ],
    },
}]
```

Components in Django template directories become available by name:

```django
{% hyper Greeting name=user.first_name / %}
```

## Use Jinja

Install the Jinja extra:

```bash
uv add "hyperhtml[jinja2]"
```

Add the extension to an environment with a filesystem loader:

```python
from jinja2 import Environment, FileSystemLoader

from hyperhtml.integrations.jinja2 import HyperExtension


env = Environment(
    loader=FileSystemLoader("templates"),
    extensions=[HyperExtension],
)
```

`.hyper` files under `templates/` become Jinja globals:

```jinja
{{ Greeting(name="Ada") }}
```

Both Jinja and Django preserve Hyper output as safe HTML. Hyper still escapes values passed into components.

## Compose components

Components compose like HTML elements:

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
from app.components import ProductCard
from app.models import Product

products: list[Product]
---
<main>
    for product in products:
        <{ProductCard}
            name={product.name}
            price={product.price}
            image={product.image}
            on_sale={product.on_sale}
        />
    end
</main>
```

## Use control flow

The template body accepts Python control flow. Blocks end with `end`:

```hyper
from app.enums import Status
from app.models import Product

status: Status
products: list[Product]
---
match status:
    case Status.LOADING:
        <div class="spinner"></div>
    case Status.ERROR:
        <p class="error">Something went wrong</p>
    case Status.OK:
        if not products:
            <p>No products found</p>
        else:
            for product in products:
                <p>{product.name}</p>
            end
        end
end
```

## Stream a response

Every component exposes its generated function as `.stream`:

```python
from fastapi.responses import StreamingResponse

from app.pages import Store


@app.get("/store")
def store():
    return StreamingResponse(
        Store.stream(products=products),
        media_type="text/html",
    )
```

## IDE support

- **JetBrains:** syntax highlighting, Python language injection, and compiler diagnostics
- **TextMate and VS Code:** syntax highlighting

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
