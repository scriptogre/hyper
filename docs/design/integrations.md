# Integrations

A Hyper component is just a function that returns HTML.

Take `Greeting.hyper`:

```hyper
name: str
---
<h1>Hello {name}</h1>
```

Under the hood, this is equivalent to a Python function:

```python
def Greeting(name: str) -> str: ...
```

To use it from a template, call the function. That is the whole integration.

## Jinja2

1. Add the extension to your `Environment`:

    ```python
    from jinja2 import Environment, FileSystemLoader

    env = Environment(loader=FileSystemLoader("templates"))
    env.add_extension("hyper.integrations.jinja2.HyperExtension")
    # env.register_components(myapp.other.path.to.components)  # register components outside templates/
    ```

2. Add `Greeting.hyper` to the `templates/` folder.

3. Call it by name in any template:

    ```jinja
    {{ Greeting(name="Ada") }}      {# → <h1>Hello Ada</h1> #}
    ```

`Card.hyper` has a default slot (`{...}`) and a named `<{...actions}>` slot:

```hyper
title: str

---

<section class="card">
    <h2>{title}</h2>

    # Default slot
    {...}

    <footer>
        # Named slot
        <{...actions}>
            <span>No actions</span>
        </{...actions}>
    </footer>
</section>
```

Fill them by wrapping the call in `{% hyper %}`:

```jinja
{% hyper Card(title="Pricing") %}
    <p>Three tiers, no surprises.</p>
    {% slot actions %}<a href="/buy">Buy now</a>{% endslot %}
{% endhyper %}
```

```html
<section class="card">
    <h2>Pricing</h2>
    <p>Three tiers, no surprises.</p>
    <footer><a href="/buy">Buy now</a></footer>
</section>
```

Spread a dict with `**`, like Python:

```jinja
{{ Card(**props) }}
{% hyper Card(title="Pricing", **props) %}…{% endhyper %}
```

## Django

1. Add the app to `INSTALLED_APPS`:

    ```python
    INSTALLED_APPS = [
        ...,
        "hyper.integrations.django",
    ]
    ```

2. Register the tag as a builtin, so you skip `{% load hyper %}`:

    ```python
    TEMPLATES = [{
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "OPTIONS": {"builtins": ["hyper.integrations.django.templatetags.hyper"]},
    }]
    ```

3. Add `Greeting.hyper` to any `templates/` folder. Hyper finds it wherever Django looks for templates (each app's `templates/`, plus the backend's `DIRS`).

4. Call a component:

    ```django
    {% hyper Greeting name=user.first_name / %}
    ```

    → `<h1>Hello Ada</h1>`

A trailing `/` self-closes a no-slot tag (note the space before `%}`).

To fill slots, drop the `/` and close with `{% endhyper %}`:

```django
{% hyper Card title="Pricing" %}
    <p>Three tiers, no surprises.</p>
    {% slot actions %}<a href="/buy">Buy now</a>{% endslot %}
{% endhyper %}
```

```html
<section class="card">
    <h2>Pricing</h2>
    <p>Three tiers, no surprises.</p>
    <footer><a href="/buy">Buy now</a></footer>
</section>
```

Spread a dict with `**`, like Python:

```django
{% hyper Card title="Pricing" **props %}…{% endhyper %}
```

### Using Django's Jinja2 backend

1. Add the extension to the Jinja2 backend's `OPTIONS`:

    ```python
    TEMPLATES = [{
        "BACKEND": "django.template.backends.jinja2.Jinja2",
        "DIRS": [BASE_DIR / "templates"],
        "APP_DIRS": True,
        "OPTIONS": {"extensions": ["hyper.integrations.jinja2.HyperExtension"]},
    }]
    ```

2. Call components with the Jinja syntax from above.

Both backends run side by side without clashing.

## FastAPI

1. Return a component. Set `response_class` to `HTMLResponse`:

    ```python
    from fastapi.responses import HTMLResponse

    @app.get("/", response_class=HTMLResponse)
    def index():
        return Greeting(name="Ada")
    ```

2. Stream with `StreamingResponse`:

    ```python
    from fastapi.responses import StreamingResponse

    @app.get("/stream")
    def stream():
        return StreamingResponse(Greeting.stream(name="Ada"), media_type="text/html")
    ```

## Litestar

1. Return a component. Set `media_type` to `MediaType.HTML`:

    ```python
    from litestar import get, MediaType

    @get("/", media_type=MediaType.HTML)
    async def index() -> str:
        return Greeting(name="Ada")
    ```

2. Stream with `Stream`:

    ```python
    from litestar import get, MediaType
    from litestar.response import Stream

    @get("/stream", media_type=MediaType.HTML)
    async def stream() -> Stream:
        return Stream(Greeting.stream(name="Ada"))
    ```

## Sanic

1. Return a component with `response.html`:

    ```python
    from sanic import response

    @app.get("/")
    async def index(request):
        return response.html(Greeting(name="Ada"))
    ```

2. Stream with `ResponseStream`:

    ```python
    from sanic.response import ResponseStream

    @app.get("/stream")
    async def stream(request):
        async def body(res):
            for chunk in Greeting.stream(name="Ada"):
                await res.write(chunk)
        return ResponseStream(body, content_type="text/html")
    ```

## Flask

1. Return a component. A returned string is already HTML:

    ```python
    @app.get("/")
    def index():
        return Greeting(name="Ada")
    ```

2. Stream any iterator:

    ```python
    @app.get("/stream")
    def stream():
        return Greeting.stream(name="Ada")
    ```

