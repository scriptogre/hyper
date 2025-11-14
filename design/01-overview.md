# Overview

**HyperProse** is a hypermedia-driven web framework for Python 3.14 built on Starlette and tdom.

Hyper (what: hypermedia) + Prose (how: like prose) = HyperProse

---

## Philosophy

HyperProse is designed around three core principles:

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
pip install hyperprose
```

Requires Python 3.14+

---

## Quick Start

### 1. Create a route

```python
# routes/index.py
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
hyperprose dev
```

Visit `http://localhost:8000` - that's it!

---

## Core Concepts

### File-Based Routing

Your file structure automatically maps to URLs:

```
routes/
  index.py          → /
  about.py          → /about
  users/
    {user_id}.py    → /users/{user_id}
```

See [02-routing.md](02-routing.md) for details.

### T-String Templates

Routes use Python 3.14's t-string templates:

```python
# routes/index.py
name = "World"

t"""
<html>
<body>
    <h1>Hello, {name}!</h1>
</body>
</html>
"""
```

See [03-templates.md](03-templates.md) for layouts and components.

### Dependency Injection

Type hints automatically inject values:

```python
# routes/users/{user_id}.py
user_id: int  # Automatically injected from URL

user = get_user(user_id)

t"""<html><body><h1>{user.name}</h1></body></html>"""
```

See [05-dependency-injection.md](05-dependency-injection.md) for all injection patterns.

### Fragments (For HTMX)

Mark elements with `{fragment}` to enable partial page updates - no duplication needed:

```python
# routes/users.py
from layouts import Base

users = get_users()

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

Perfect for HTMX partial updates without writing separate endpoints!

See [04-fragments.md](04-fragments.md) for details.

---

## What's Next?

- **[Routing](02-routing.md)** - Learn about file-based routing
- **[Templates](03-templates.md)** - Build layouts and components
- **[Fragments](04-fragments.md)** - Enable partial page updates
- **[Dependency Injection](05-dependency-injection.md)** - Master type-based injection

---

**[← Back to Index](README.md)** | **[Next: Routing →](02-routing.md)**
