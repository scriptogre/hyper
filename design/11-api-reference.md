# API Reference

---

## Imports

```python
from hyper import (
    # Core
    Hyper,            # Main application class

    # Request/Response
    Request,           # Starlette Request
    Response,          # Starlette Response
    HTMLResponse,      # HTML response
    JSONResponse,      # JSON response
    RedirectResponse,  # Redirect response
    StreamingResponse, # Streaming response

    # HTTP Method Helpers
    GET,               # Check if request is GET
    POST,              # Check if request is POST
    PUT,               # Check if request is PUT
    DELETE,            # Check if request is DELETE
    PATCH,             # Check if request is PATCH

    # Dependency Injection
    Query,             # Query parameter config
    Header,            # Header injection
    Cookie,            # Cookie injection
    Form,              # Form field injection
    Body,              # Request body injection
    File,              # File upload
    UploadFile,        # File upload type

    # Markdown
    MarkdownMeta,      # Markdown frontmatter type
    get_collection,    # Get markdown files
    render_markdown,   # Render markdown string

    # Partials
    Fragment,          # Fragment tag
    partial,           # Partial attribute
)
```

---

## Hyper Class

```python
app = Hyper(
    pages_dir: str = "pages",
    static_dir: str = "static",
    debug: bool = False,
    templates: dict = {
        "trim_newlines": True,  # Remove newlines after expressions
        "trim_indent": True,    # Remove leading indentation
    },
)
```

**Parameters:**
- `pages_dir` - Directory containing page files (default: `"pages"`)
- `static_dir` - Directory for static files (default: `"static"`)
- `debug` - Enable debug mode (default: `False`)
- `templates` - Template configuration (whitespace control, etc.)

**Methods:**
- `add_middleware(middleware_class, **options)` - Add Starlette middleware
- `exception_handler(status_code)` - Decorator for custom error handlers

---

## Lifespan Events

Handle startup and shutdown logic using the `lifespan` parameter, just like FastAPI:

```python
# app/main.py
from contextlib import asynccontextmanager
from hyper import Hyper

# Shared resources
ml_models = {}

@asynccontextmanager
async def lifespan(app: Hyper):
    # Startup: Load resources before handling requests
    ml_models["model"] = load_ml_model()
    print("Application started")

    yield  # Application runs and handles requests

    # Shutdown: Clean up resources after handling requests
    ml_models.clear()
    print("Application stopped")

app = Hyper(lifespan=lifespan)
```

**For complex scenarios**, extract to `app/lifecycle.py`:

```python
# app/lifecycle.py
from contextlib import asynccontextmanager

@asynccontextmanager
async def lifespan(app):
    # Startup
    await init_database()
    await load_cache()
    yield
    # Shutdown
    await close_database()
    await clear_cache()
```

```python
# app/main.py
from hyper import Hyper
from app.lifecycle import lifespan

app = Hyper(lifespan=lifespan)
```

**When to use:**
- Database connection pools
- Loading ML models or caching data
- Background task setup/cleanup
- Resource initialization that should happen once

---

## Project Structure

```
my_app/
├── main.py                   # Application entry point
├── app/
│   ├── pages/                # Hypermedia pages, layouts, partials
│   │   ├── Base.py           # Base layout (PascalCase = not a route)
│   │   ├── AuthLayout.py     # Auth layout (PascalCase = not a route)
│   │   ├── index.py          # /
│   │   ├── about.py          # /about
│   │   ├── contact.py        # /contact
│   │   ├── users/
│   │   │   ├── index.py      # /users
│   │   │   ├── [id]/
│   │   │   │   ├── index.py  # /users/{id}
│   │   │   │   ├── Form.py   # Partial (PascalCase = not a route)
│   │   │   │   └── Avatar.py # Partial (PascalCase = not a route)
│   │   │   └── create.py     # /users/create
│   │   └── blog/
│   │       ├── BlogLayout.py # Blog layout (PascalCase = not a route)
│   │       ├── index.py      # /blog
│   │       └── [slug].py     # /blog/{slug}
│   ├── api/                  # JSON endpoints (optional)
│   │   └── posts/
│   │       ├── index.py      # /api/posts
│   │       └── [id].py       # /api/posts/{id}
│   ├── models/               # Database models
│   │   ├── user.py
│   │   └── post.py
│   ├── services/             # Business logic
│   │   ├── auth.py
│   │   └── email.py
│   └── schemas/              # Validation schemas
│       ├── user.py
│       └── post.py
├── components/               # Shared, stateless UI (only after 3+ uses)
│   ├── Button.py
│   ├── Card.py
│   └── Modal.py
├── static/
│   ├── css/
│   │   └── style.css
│   ├── js/
│   │   └── app.js
│   └── images/
│       └── logo.png
└── .env                      # Environment variables
```

---

## Comparison with Other Frameworks

### Hyper vs FastAPI

| Feature       | Hyper                   | FastAPI             |
|---------------|-------------------------|---------------------|
| **Focus**     | Server-side HTML        | REST APIs           |
| **Routing**   | File-based              | Decorator-based     |
| **Templates** | Built-in (tdom)         | External (Jinja2)   |
| **DI Style**  | Module-level type hints | Function parameters |
| **Best for**  | Hypermedia apps         | JSON APIs           |

### Hyper vs Django

| Feature       | Hyper            | Django                   |
|---------------|------------------|--------------------------|
| **Templates** | Python t-strings | Django Template Language |
| **Routing**   | File-based       | urls.py config           |
| **ORM**       | BYO              | Built-in                 |
| **Admin**     | None             | Built-in                 |
| **Best for**  | Modern HTMX apps | Traditional MVC apps     |

### Hyper vs Flask

| Feature       | Hyper              | Flask            |
|---------------|--------------------|------------------|
| **Async**     | Native (Starlette) | Optional (Quart) |
| **Routing**   | File-based         | Decorator-based  |
| **Templates** | T-strings (tdom)   | Jinja2           |
| **DI**        | Type hints         | Manual           |
| **Best for**  | Async hypermedia   | Sync traditional |

---

## Tips & Best Practices

### 1. Use PascalCase for Non-Routes

Files with PascalCase names are automatically excluded from routing:
- `Base.py` - Layouts
- `Form.py` - Partials
- `Button.py` - Components

No prefixes or special markers needed - the naming itself signals intent.

### 2. Keep Routes Simple

Do heavy logic in separate modules:

```python
# app/pages/users/index.py
from app.services.users import get_all_users_with_stats
from app.pages import Base

users = get_all_users_with_stats()

t"""
<{Base} title="Users">
    {[t'<div>{u.name} - {u.post_count} posts</div>' for u in users]}
</{Base}>
"""
```

### 3. Avoid Premature Extraction

**Keep HTML inline as long as possible:**
- Don't extract during active development
- Don't extract coupled elements (e.g., `hx-target` pairs)
- Inline loops are often clearer than components

```python
# ✅ GOOD - Clear and inline
t"""
<{Base}>
    {[t'<article class="post-card">{post.title}</article>' for post in posts]}
</{Base}>
"""

# ❌ PREMATURE - Unnecessary abstraction
t"""
<{Base}>
    {[t"<{PostCard} post={post} />" for post in posts]}
</{Base}>
"""
```

**Only extract when:**
1. Route file becomes truly cluttered (200+ lines)
2. After copy-pasting across 3+ routes (→ `/app/components`)

Premature extraction makes projects harder to work with.

### 4. Use Directory Routes for Complex Pages

When a route needs partials, make it a directory:

```python
# Simple route (single file)
users/create.py

# Complex route (directory with partials)
users/[id]/
  index.py      # Main route
  Form.py       # Partial
  Avatar.py     # Partial
```

### 5. Leverage HTMX for Interactivity

Return partials for dynamic updates:

```python
# app/pages/users/[id]/follow.py
from hyper import POST

id: int

if POST:
    follow_user(id)
    follower_count = get_follower_count(id)

    # Return just the updated button
    t"""
    <button hx-post="/users/{id}/unfollow" hx-swap="outerHTML">
        Unfollow ({follower_count} followers)
    </button>
    """
```

### 6. Use Layouts for Consistency

Define layouts once, use everywhere:

```python
# app/pages/Base.py
title: str = "My App"

t"""
<!doctype html>
<html lang="en">
<head>
    <title>{title}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="/static/css/style.css">
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body>
    <header>
        <nav>
            <a href="/">Home</a>
            <a href="/about">About</a>
        </nav>
    </header>
    <main>{...}</main>
    <footer>&copy; 2025</footer>
</body>
</html>
"""
```

---

## Troubleshooting

<!-- TODO: Add common issues as they arise during actual usage -->

---

## Design Principles

- **File-based routing** - Your file structure IS your routing configuration
- **Hypermedia-first** - Built for server-rendered HTML and HTMX
- **Type-based injection** - Module-level type hints for dependencies
- **No decorators needed** - Convention over configuration
- **Inline by default** - Extract only when necessary
- **Native Python** - T-strings (3.14+), async/await throughout

---

## Resources

- **Hyper Docs:** (coming soon)
- **tdom Documentation:** https://github.com/thoughtbot/tdom
- **Starlette Documentation:** https://www.starlette.io
- **HTMX Documentation:** https://htmx.org
- **FastAPI Documentation:** https://fastapi.tiangolo.com (for injection patterns)
- **Python 3.14 Release Notes:** https://docs.python.org/3.14/whatsnew/3.14.html

---

**Happy building with Hyper! 🚀**

---

**[← Previous: Advanced Features](10-advanced.md)** | **[Back to Index](README.md)**
