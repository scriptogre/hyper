# HyperProse

**A hypermedia-driven web framework for Python 3.14 built on Starlette and tdom**

Hyper (what: hypermedia) + Prose (how: like prose) = HyperProse

Write server-rendered HTML apps with the simplicity of modern Python. File-based routing meets FastAPI-style dependency injection, powered by native t-string templates.

```python
# routes/index.py
from hyperprose import GET, POST, Form
from typing import Annotated
from layouts import Base

if GET:
    t"""
    <{Base} title="Welcome">
        <h1>Hello, HyperProse!</h1>
        <form method="POST">
            <input name="message" placeholder="Enter a message">
            <button>Submit</button>
        </form>
    </{Base}>
    """

elif POST:
    message: Annotated[str, Form()]
    t"""
    <{Base} title="Success">
        <h2>You said: {message}</h2>
    </{Base}>
    """
```

---

## Key Features

### 🗂️ File-Based Routing
Your file structure **is** your URL structure. No decorators, no configuration.

```
routes/
  index.py              → /
  about.py              → /about
  blog/
    index.py            → /blog
    {slug}.py           → /blog/{slug}
```

### 🎨 Native T-String Templates
Python 3.14's t-strings with tdom provide type-safe, compiled templates with zero learning curve.

```python
# Advanced attribute handling
classes = ["btn", "btn-primary", {"active": is_admin}]
styles = {"color": "red", "margin": "10px"}

t"""
<button
    class={classes}
    style={styles}
    data={{"user-id": 123}}
    {_if(is_visible)}>
    Click Me
</button>
"""
```

### 🎯 Conditional Rendering
Clean syntax for conditional elements:

```python
from hyperprose import _if

# Block syntax for multiple elements
t"""
{_if(show_sidebar):}
    <aside>{sidebar}</aside>
{_if:end}
"""

# Inline syntax for single elements
t"""
<a href="/admin" {_if(is_admin)}>Admin Panel</a>
"""
```

### 💉 FastAPI-Style Dependency Injection
Module-level type hints automatically inject what you need:

```python
from hyperprose import GET, POST, Form
from typing import Annotated

user_id: int  # Path parameter from URL
page: int = 1  # Query parameter with default
user: User = Depends(get_current_user)  # Custom dependency

if POST:
    name: Annotated[str, Form()]  # Form field
    email: Annotated[str, Form()]
```

### 🧩 Component System
Layouts and components with slots—no JSX, just Python:

```python
# layouts/Base.py
from hyperprose import Slot

title: str = "My App"
content: Slot

t"""
<!doctype html>
<html>
<head><title>{title}</title></head>
<body>
    <nav>...</nav>
    <main>{content}</main>
</body>
</html>
"""
```

```python
# routes/index.py
from layouts import Base

t"""
<{Base} title="Home">
    <h1>Welcome!</h1>
</{Base}>
"""
```

### ⚡ HTMX-Ready
Perfect for hypermedia-driven applications:

```python
from hyperprose import POST

user_id: int

if POST:
    follow_user(user_id)
    count = get_follower_count(user_id)

    # Return just the updated element
    t"""
    <button hx-post="/users/{user_id}/unfollow" hx-swap="outerHTML">
        Unfollow ({count} followers)
    </button>
    """
```

### 📝 Forms Made Easy
Handle forms with automatic field extraction and validation:

```python
from hyperprose import GET, POST, Form
from typing import Annotated
from pydantic import BaseModel, EmailStr

class SignupForm(BaseModel):
    username: str
    email: EmailStr
    password: str

if GET:
    t"""<form method="POST">...</form>"""

elif POST:
    form: Annotated[SignupForm, Form()]
    user = create_user(form.username, form.email, form.password)
    t"""<h1>Welcome, {user.username}!</h1>"""
```

### 🔄 Fragments & Streaming
Render page sections independently with inline fragment markers:

```python
from hyperprose import fragment, render

user_id: int
user = User.get(id=user_id)

t"""
<html>
<body>
    <h1>{user.name}</h1>

    <div id="status" {fragment}>
        Status: {user.status}
        Last updated: {user.updated_at}
    </div>

    <div id="activity" {fragment}>
        Recent activity: {user.recent_activity}
    </div>
</body>
</html>
"""

# Render only specific fragments
render(fragments=["status"])

# Stream real-time updates
async def stream():
    yield render(fragments="status")
    async for update in subscribe_to_updates(user_id):
        yield render(fragments="status")
```

### 📄 Markdown-First
Full Python execution in markdown files with frontmatter and inline Python:

```python
# routes/blog/{slug}.md
# ---
# title: My Blog Post
# date: 2025-01-15
# ---
#
# # {frontmatter.title}
# Published on {frontmatter.date}
#
# Python code blocks execute and populate variables:
# posts = get_recent_posts(limit=5)
#
# ## Recent Posts
# {[t'<li><a href="/blog/{p.slug}">{p.title}</a></li>' for p in posts]}
```

### 🏗️ SSG + SSR Hybrid
Static site generation with dynamic capabilities:

```python
# Pre-render at build time
@staticmethod
def get_static_paths():
    return [{"slug": post.slug} for post in get_all_posts()]
```

---

## Table of Contents

### Core Concepts

1. **[Overview](design/01-overview.md)** - Philosophy, installation, and quick start
2. **[Routing](design/02-routing.md)** - File-based routing and route structure
3. **[Templates & Layouts](design/03-templates.md)** - T-string templates, layouts, and components
4. **[Fragments](design/04-fragments.md)** - Inline fragments for partial page updates
5. **[Dependency Injection](design/05-dependency-injection.md)** - FastAPI-style injection patterns

### Features

6. **[Forms](design/06-forms.md)** - Form handling with automatic field extraction
7. **[Streaming & SSE](design/07-streaming.md)** - Server-Sent Events and async/await
8. **[Static Site Generation](design/08-ssg.md)** - SSG + SSR hybrid mode
9. **[Markdown](design/09-markdown.md)** - First-class markdown support

### Advanced

10. **[Advanced Features](design/10-advanced.md)** - Response manipulation, HTMX, static files, error pages
11. **[API Reference](design/11-api-reference.md)** - Complete API, project structure, tips & troubleshooting

---

## Quick Links

- **GitHub:** https://github.com/scriptogre/hyperprose
- **tdom:** https://github.com/thoughtbot/tdom
- **Starlette:** https://www.starlette.io
- **htmx:** https://htmx.org

---

## What Makes HyperProse Different?

- **File-based routing** - Your file structure IS your URL structure
- **T-string templates** - Native Python 3.14 templates with full type safety
- **FastAPI-style injection** - Module-level type hints for automatic dependency injection
- **Inline fragments** - Render page sections without duplication
- **SSG + SSR hybrid** - Static when possible, dynamic when needed
- **Streaming built-in** - Real-time updates with Server-Sent Events
- **Markdown-first** - Full Python execution in markdown files

---

## Getting Started

Start with [01-overview.md](design/01-overview.md) for installation and your first route, then explore the other guides based on what you're building.
