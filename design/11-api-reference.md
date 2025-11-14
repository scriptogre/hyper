# API Reference

---

## Imports

```python
from hyperprose import (
    # Core
    HyperProse,            # Main application class

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
    Children,          # For layout components
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

    # tdom Types (re-exported)
    Node,              # Base node type
    Element,           # Element node
    Markup,            # Safe HTML wrapper
    Slot,              # Slot type for component content
    _if,               # Conditional rendering helper

    # HTMX Helpers
    is_htmx,           # Check if HTMX request
    hx_redirect,       # HX-Redirect helper
    hx_trigger,        # HX-Trigger helper

    # Fragment Support
    fragment,          # Fragment marker
    render,            # Fragment rendering
)
```

---

## HyperProse Class

```python
app = HyperProse(
    routes_dir: str = "routes",
    static_dir: str = "static",
    debug: bool = False,
)
```

**Parameters:**
- `routes_dir` - Directory containing route files (default: `"routes"`)
- `static_dir` - Directory for static files (default: `"static"`)
- `debug` - Enable debug mode (default: `False`)

**Methods:**
- `add_middleware(middleware_class, **options)` - Add Starlette middleware
- `add_event_handler(event_type, func)` - Add startup/shutdown handlers
- `exception_handler(status_code)` - Decorator for custom error handlers
- `on_event(event_type)` - Decorator for lifecycle events

---

## Project Structure

```
my_app/
├── app.py                    # Application entry point
├── routes/
│   ├── _base.py              # Base layout
│   ├── _components.py        # Shared components
│   ├── index.py              # /
│   ├── about.py              # /about
│   ├── contact.py            # /contact
│   ├── users/
│   │   ├── index.py          # /users
│   │   ├── {user_id}.py      # /users/{user_id}
│   │   └── {user_id}/
│   │       ├── edit.py       # /users/{user_id}/edit
│   │       └── posts/
│   │           └── {post_id}.py  # /users/{user_id}/posts/{post_id}
│   ├── blog/
│   │   ├── _layout.py        # Blog layout
│   │   ├── index.py          # /blog
│   │   └── {slug}.py         # /blog/{slug}
│   └── api/
│       ├── users.py          # /api/users
│       └── posts.py          # /api/posts
├── static/
│   ├── css/
│   │   └── style.css
│   ├── js/
│   │   └── app.js
│   └── images/
│       └── logo.png
├── models.py                 # Database models
├── database.py               # Database connection
└── utils.py                  # Helper functions
```

---

## Comparison with Other Frameworks

### HyperProse vs FastAPI

| Feature | HyperProse | FastAPI |
|---------|--------|---------|
| **Focus** | Server-side HTML | REST APIs |
| **Routing** | File-based | Decorator-based |
| **Templates** | Built-in (tdom) | External (Jinja2) |
| **DI Style** | Module-level type hints | Function parameters |
| **Best for** | Hypermedia apps | JSON APIs |

### HyperProse vs Django

| Feature | HyperProse | Django |
|---------|--------|--------|
| **Templates** | Python t-strings | Django Template Language |
| **Routing** | File-based | urls.py config |
| **ORM** | BYO | Built-in |
| **Admin** | None | Built-in |
| **Best for** | Modern HTMX apps | Traditional MVC apps |

### HyperProse vs Flask

| Feature | HyperProse | Flask |
|---------|--------|-------|
| **Async** | Native (Starlette) | Optional (Quart) |
| **Routing** | File-based | Decorator-based |
| **Templates** | T-strings (tdom) | Jinja2 |
| **DI** | Type hints | Manual |
| **Best for** | Async hypermedia | Sync traditional |

---

## Tips & Best Practices

### 1. Organize with Underscored Files

Use `_` prefix for non-route files:
- `_base.py` - Layouts
- `_components.py` - Components
- `_utils.py` - Utilities
- `_models.py` - Data models

### 2. Keep Routes Simple

Do heavy logic in separate modules:

```python
# routes/users/index.py
from services.users import get_all_users_with_stats
from routes._base import Layout

users = get_all_users_with_stats()

t"""
<{Layout} title="Users">
    {[t'<div>{u.name} - {u.post_count} posts</div>' for u in users]}
</{Layout}>
"""
```

### 3. Use Components for Reusability

Extract common patterns into components:

```python
# routes/_components.py
def Card(*, title, children=(), **attrs):
    return t"""
    <div class="card" {attrs}>
        <h3>{title}</h3>
        <div class="card-body">{children}</div>
    </div>
    """
```

### 4. Leverage HTMX for Interactivity

Return partials for dynamic updates:

```python
# routes/users/{user_id}/follow.py
from hyperprose import POST

user_id: int

if POST:
    follow_user(user_id)
    follower_count = get_follower_count(user_id)

    # Return just the updated button
    t"""
    <button hx-post="/users/{user_id}/unfollow" hx-swap="outerHTML">
        Unfollow ({follower_count} followers)
    </button>
    """
```

### 5. Use Layouts for Consistency

Define layouts once, use everywhere:

```python
# routes/_base.py
from hyperprose import Children

children: Children
title: str = "My App"

Layout = t"""
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
    <main>{children}</main>
    <footer>&copy; 2025</footer>
</body>
</html>
"""
```

---

## Troubleshooting

### Template Not Found

**Error:** `No template found in module`

**Solution:** Make sure you have a bare t-string in your route file:

```python
# ✅ Correct
t"""<html>...</html>"""

# ❌ Wrong - variable assignment
page = t"""<html>...</html>"""
```

### Path Parameter Not Injected

**Error:** `NameError: name 'user_id' is not defined`

**Solution:** Add type hint:

```python
# ✅ Correct
user_id: int
user = get_user(user_id)

# ❌ Wrong - no type hint
user = get_user(user_id)  # user_id not injected!
```

### Query Parameter Not Working

**Error:** Query param not populated

**Solution:** Add type hint AND default value:

```python
# ✅ Correct
limit: int = 10

# ❌ Wrong - no default
limit: int  # Treated as required path param!
```

### Async Function Not Called

**Error:** Async function defined but data not loaded

**Solution:** Make sure function is actually async and uses `global`:

```python
# ✅ Correct
user: User

async def load():
    global user
    user = await get_user(user_id)

# ❌ Wrong - not using global
async def load():
    user = await get_user(user_id)  # Local variable!
```

### Form Fields Not Injected

**Error:** Form fields undefined in POST handler

**Solution:** Use `Annotated[type, Form()]` and check with `POST`:

```python
# ✅ Correct
from typing import Annotated
from hyperprose import POST, Form

if POST:
    name: Annotated[str, Form()]
    # Use name here

# ❌ Wrong - no method check
name: Annotated[str, Form()]  # Will fail on GET!
```

---

## Philosophy & Design Decisions

### Why No Decorators?

File-based routing eliminates the need for route decorators. Your file structure IS your routing configuration.

### Why Type Hints for DI?

Type hints are explicit, tooling-friendly, and Pythonic. They provide:
- IDE autocomplete
- Type checking
- Self-documenting code
- No magic strings

### Why T-Strings?

T-strings are:
- Native Python (3.14+)
- Type-safe
- Fast (compiled)
- Familiar (like f-strings)
- Powerful (via tdom)

### Why No `page =` Variable?

Less boilerplate = better DX. The framework is smart enough to find the template in your module.

### Why Starlette?

Starlette provides:
- High performance
- Full async support
- Proven stability
- Rich ecosystem
- WebSocket support
- Middleware system

---

## Resources

- **HyperProse Docs:** (coming soon)
- **tdom Documentation:** https://github.com/thoughtbot/tdom
- **Starlette Documentation:** https://www.starlette.io
- **HTMX Documentation:** https://htmx.org
- **FastAPI Documentation:** https://fastapi.tiangolo.com (for injection patterns)
- **Python 3.14 Release Notes:** https://docs.python.org/3.14/whatsnew/3.14.html

---

**Happy building with HyperProse! 🚀**

---

**[← Previous: Advanced Features](10-advanced.md)** | **[Back to Index](README.md)**
