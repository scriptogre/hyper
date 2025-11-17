# Overview

**Hyper** is a hypermedia-driven web framework for Python 3.14 built on Starlette, FastAPI, and tdom.

You could also think of it as "Astro" for Python, optimized for server-rendered HTML and HTMX.

---

## Philosophy

Hyper is designed around three core principles:

1. **Write like prose** - Your code should read naturally, with minimal boilerplate and maximum clarity
2. **Convention over configuration** - Smart defaults and intuitive patterns mean you spend less time configuring and more time building
3. **Hypermedia-first** - Built for server-rendered HTML and HTMX, not JSON APIs

---

## Key Features

- **File-based routing** - Your folder structure is your URL structure
- **T-string templates** - Use Python 3.14's native template strings
- **Type-based injection** - FastAPI-style dependency injection via type hints
- **Inline fragments** - Render page sections without duplication (perfect for HTMX)
- **SSG + SSR hybrid** - Static when you want it, dynamic when you need it
- **Streaming & SSE** - Real-time updates built-in
- **Markdown support** - Write content-heavy pages in markdown with full Python power
- **Async throughout** - Native async/await support everywhere

---

## Installation

```bash
pip install hyper
```

Requires Python 3.14+

---

## Quick Start

### 1. Create a route

```python
# app/pages/index.py
t"""
<!doctype html>
<html>
<head><title>Home</title></head>
<body>
    <h1>Hello, World!</h1>
</body>
</html>
"""
```

### 2. Run it

```bash
hyper dev
```

Visit `http://localhost:8000` - that's it!

---

## Configuration (Optional)

Hyper works with zero configuration. But if you need to customize settings, create a `main.py`:

```python
# main.py
from hyper import Hyper

app = Hyper(
    templates={
        "trim_newlines": True,  # Default: removes newlines after expressions
        "trim_indent": True,    # Default: removes leading indentation
    }
)
```

Then run:
```bash
hyper dev
```

Hyper automatically detects and uses `main.py` if it exists.

<details>
<summary><strong>What do these settings do?</strong></summary>

These settings control how whitespace in your templates is rendered:

- **`trim_newlines`**: Removes the first newline after template expressions and blocks
- **`trim_indent`**: Removes leading spaces/tabs from lines containing template expressions

With both enabled (default), your nicely-indented templates produce clean HTML without extra blank lines or indentation artifacts.

See [Whitespace Control](03-templates.md#whitespace-control) for detailed examples.

</details>

---

## Core Concepts

### File-Based Routing

Your file structure automatically maps to URLs:

```
app/pages/
  index.py          → /
  about.py          → /about
  users/
    [id].py         → /users/{id}
```

See [02-routing.md](02-routing.md) for details.

### T-String Templates

Routes use Python 3.14's t-string templates:

```python
# app/pages/index.py
name = "World"

t"""
<html>
<body>
    <h1>Hello, {name}!</h1>
</body>
</html>
"""
```

See [03-templates.md](03-templates.md) for layouts, partials, and components.

### Dependency Injection

Type hints automatically inject values:

```python
# app/pages/users/[id].py
from app.models import User

id: int  # Injected from URL
user = User.get(id=id)

t"""<html><body><h1>{user.name}</h1></body></html>"""
```

See [05-dependency-injection.md](05-dependency-injection.md) for all injection patterns.

### Fragments (For HTMX)

Mark elements with `{fragment}` to enable partial page updates - no duplication needed:

```python
# app/pages/users/index.py
from hyper import fragment
from app.pages import Base

users = User.all()

t"""
<{Base}>
    <h1>Users</h1>
    <div id="user-list" {fragment}>
        {[t'<div>{u.name}</div>' for u in users]}
    </div>
</{Base}>
"""
```

**Full page request:** `/users` → Returns complete page with layout

**Fragment request:** `/users?_fragment=user-list` → Returns just the `#user-list` div

Perfect for htmx partial updates without writing separate endpoints!

See [Templates & Fragments](03-templates.md#fragments) for details.

---

## What's Next?

- **[Routing](02-routing.md)** - Learn about file-based routing
- **[Templates](03-templates.md)** - Build layouts and components
- **[Fragments](03-templates.md#fragments)** - Enable partial page updates
- **[Dependency Injection](05-dependency-injection.md)** - Master type-based injection

---

**[← Back to Index](README.md)** | **[Next: Routing →](02-routing.md)**
