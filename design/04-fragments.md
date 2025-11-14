# Fragments

Fragments let you render just part of a page, similar to `django-render-block` or `jinja2-fragments`. Everything stays inline and readable - no duplication needed.

---

## Basic Usage

Use `{fragment}` to mark any element as a fragment:

```python
# app/routes/users.py
from hyperprose import Request, fragment
from app.layouts.base import Layout

request: Request
users = get_users()

t"""
<{Layout}>
    <h1>Users</h1>

    <div id="user-list" {fragment}>
        <ul>
            {[t'<li>{u.name}</li>' for u in users]}
        </ul>
    </div>
</{Layout}>
"""
```

**How it works:**

By default, `{fragment}` uses the element's `id` as the fragment name. 

You can override the fragment name with `{fragment:custom-name}`:

```python
# app/routes/users/{user_id}.py
from hyperprose import Request, fragment
from app.layouts.base import Layout

request: Request
user_id: int
user = get_user(user_id)

t"""
<{Layout}>
    <h1>{user.name}'s Profile</h1>

    <div {fragment:profile-info}>
        <p>Email: {user.email}</p>
        <p>Joined: {user.created_at}</p>
    </div>

    <form {fragment:edit-user-{user.id}} method="POST" action="/users/{user.id}">
        <input name="email" value="{user.email}">
        <button>Update</button>
    </form>
</{Layout}>
"""
```

**Notes:**
- Supports dynamic names: `{fragment:edit-user-{user.id}}` → `id="edit-user-123"`

---

## Server-Side Fragment Control

Control which fragments render from your route logic:

```python
from typing import Annotated
from hyperprose import Request, Header, render, fragment
from app.layouts.base import Layout

request: Request
is_htmx: Annotated[str | None, Header("HX-Request")]

users = User.all()

t"""
<{Layout}>
    <h1>Users</h1>
    <div id="user-list" {fragment}>
        <ul>{[t'<li>{user.name}</li>' for user in users]}</ul>
    </div>
    <div id="user-stats" {fragment}>
        Total: {len(users)} users
    </div>
</{Layout}>
"""

if is_htmx:
    render(fragments=["user-list"])  # Render only these fragments

# Otherwise framework auto-calls render() for full page
```

**How `render()` works:**
- Framework automatically calls `render()` at the end if you don't
- Call `render(fragments=[...])` to extract specific fragments
- Accesses the template from `globals()["template"]` automatically
- No duplication - same template, different rendering modes

---

## Streaming with Fragments

Combine fragments with streaming for powerful real-time updates:

```python
from hyperprose import render, fragment
from app.layouts import Layout
from app.models import User

user_id: int

user = User.get(id=user_id)

t"""
<{Layout}>
    <h1>{user.name}</h1>
    <div id="status" {fragment}>
        Status: {user.status}
        Last updated: {user.updated_at}
    </div>
</{Layout}>
"""

async def stream():
    """Automatically re-runs the top-level template logic on each iteration."""
    yield render(fragments="status")

    async for update in subscribe_to_updates(user_id):
        yield render(fragments="status")
```

**Benefits:**
- One template, multiple rendering modes
- Re-runnable logic for streaming
- No duplication
- Simple things simple, complex things possible

---

## Multiple Fragments

You can mark multiple elements as fragments in a single template:

```python
from typing import Annotated
from hyperprose import Query, fragment, render
from app.layouts.base import Layout

users = User.all()
stats = Stat.all()

t"""
<{Layout}>
    <div id="user-list" {fragment}>
        <h2>Users</h2>
        <ul>{[t'<li>{u.name}</li>' for u in users]}</ul>
    </div>

    <div id="stats" {fragment}>
        <h2>Statistics</h2>
        <p>Total users: {stats.total}</p>
        <p>Active: {stats.active}</p>
    </div>

    <div id="activity" {fragment}>
        <h2>Recent Activity</h2>
        <ul>{[t'<li>{a.text}</li>' for a in stats.recent]}</ul>
    </div>
</{Layout}>
"""

fragments: Annotated[list, Query("_fragment")] = []  # Get from request query params

# Then, with a URL like: http://localhost:8000/items/?_fragment=foo&_fragment=bar
# _fragment will be ['foo', 'bar']

if fragments:
    # Render only the requested fragment(s)
    render(fragments=fragments)
```

---

## Key Points

- **`{fragment}` uses element's `id` as fragment name**
- **`{fragment:custom-name}` for custom names**
- **`render(fragments=[...])` for server-side control**
- **Perfect for HTMX partial updates**
- **Works with streaming for real-time updates**
- **No duplication - single template, multiple modes**

---

**[← Previous: Templates](03-templates.md)** | **[Back to Index](README.md)** | **[Next: Dependency Injection →](05-dependency-injection.md)**
