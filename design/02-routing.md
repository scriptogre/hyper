# Routing

Hyper uses **file-based routing** - your file structure automatically maps to URLs.

---

## Project Structure

```
app/
  pages/         # Hypermedia pages, layouts, and partials
    Base.py             # Layout (PascalCase = not a route)
    AuthLayout.py       # Layout (PascalCase = not a route)
    index.py            → /
    about.py            → /about
    contact.py          → /contact
    users/
      index.py          → /users
      [id]/
        index.py        → /users/{id}
        Form.py         # Partial (PascalCase = not a route)
        Avatar.py       # Partial (PascalCase = not a route)
      create.py         → /users/create
  api/           # JSON endpoints (optional)
    posts/
      index.py          → /api/posts
      [id].py           → /api/posts/{id}
  models/        # Database models
  services/      # Business logic
  schemas/       # Validation schemas

components/      # Shared, stateless UI (only after 3+ uses)
  Button.py
  Card.py
  Modal.py
```

---

## Rules

### PascalCase Files Are Not Routes

Files with PascalCase names (e.g., `Base.py`, `Form.py`) are **layouts or partials**, not routes:

```
app/pages/
  Base.py           # Layout - not a route
  index.py          # Route: /
  users/
    [id]/
      index.py      # Route: /users/{id}
      Form.py       # Partial - not a route
```

The router skips PascalCase files when generating routes.

### `[param]` Creates Path Parameters

Use square brackets for dynamic URL segments:

```
app/pages/
  users/
    [id].py         → /users/123
    [id]/
      index.py      → /users/123
      posts/
        [post_id].py → /users/123/posts/456
```

Parameters are injected via type hints (see [05-dependency-injection.md](05-dependency-injection.md)).

### `index.py` Maps to Directory Path

```
app/pages/
  index.py          → /
  users/
    index.py        → /users
  blog/
    index.py        → /blog
```

### Nested Directories Create Nested Paths

```
app/pages/
  api/
    v1/
      users.py      → /api/v1/users
```

### File vs Directory Routes

**Single file** (for simple routes):
```
users/create.py     → /users/create
```

**Directory with `index.py`** (for complex routes with partials):
```
users/[id]/
  index.py          → /users/{id}
  Form.py           # Partial
  Comments.py       # Partial
```

Both map to the same URL - use directories when you need partials.

---

## Route Files

A route file is just Python code with a t-string template.

### Simple Route

```python
# routes/about.py
t"""
<!doctype html>
<html>
<head><title>About</title></head>
<body>
    <h1>About Us</h1>
    <p>We build amazing things.</p>
</body>
</html>
"""
```

**How it works:**
1. Framework executes the module
2. Finds the t-string template (a `Template` object)
3. Converts it to HTML using tdom's `html()` function
4. Returns it as a Starlette `TemplateResponse`

### Route with Logic

```python
# routes/users.py
from app.models import User

users: list[User] = User.all()  # Your database call

t"""
<!doctype html>
<html>
<head><title>Users</title></head>
<body>
    <h1>User Directory</h1>
    <ul>
        {[t'<li>{user.name} - {user.email}</li>' for user in users]}
    </ul>
</body>
</html>
"""
```

**Variables in scope are available in the template!**

### Route with Path Parameters

```python
# app/pages/users/[id].py
from app.models import User

id: int  # Automatically injected from URL path

user = User.get(id=id)

t"""
<!doctype html>
<html>
<head><title>{user.name}</title></head>
<body>
    <h1>Profile: {user.name}</h1>
    <p>Email: {user.email}</p>
    <p>User ID: {id}</p>
</body>
</html>
"""
```

**URL:** `/users/123`
**Result:** `id` is injected as `123` (converted to int)

**Note:** Use `[id]` in the filename, and `id` (without brackets) as the variable name.

### Route with Query Parameters

TODO: Replace with https://htmx.org/examples/active-search/ example

```python
# routes/search.py

q: str = ""          # ?q=...
limit: int = 10      # ?limit=...
page: int = 1        # ?page=...

results = search(q, limit=limit, page=page)

t"""
<!doctype html>
<html>
<head><title>Search: {q}</title></head>
<body>
    <h1>Search Results for "{q}"</h1>
    <p>Found {len(results)} results</p>
    <div class="results">
        {[t'<div class="result"><h3>{r.title}</h3></div>' for r in results]}
    </div>
</body>
</html>
"""
```

**URL Examples:**
- `/search` → `q=""`, `limit=10`, `page=1` (defaults)
- `/search?q=python` → `q="python"`, `limit=10`, `page=1`
- `/search?q=python&limit=20&page=2` → `q="python"`, `limit=20`, `page=2`

---

## Multiple HTTP Methods

Use `GET`, `POST`, `PUT`, `DELETE` helpers to handle different HTTP methods:

```python
# app/pages/contact.py
from hyper import GET, POST
from app.pages import Base

if GET:
    # Show the form
    t"""
    <{Base} title="Contact">
        <form method="POST">
            <input name="email" type="email">
            <button>Submit</button>
        </form>
    </{Base}>
    """

elif POST:
    # Process form
    t"""
    <{Base} title="Success">
        <h1>Thank you!</h1>
    </{Base}>
    """
```

See [06-forms.md](06-forms.md) for full form handling patterns.

---

## Directory Structure Philosophy

### Why `/app/pages`?

**`/app`** contains all application code (pages, models, services, schemas). It's your application namespace.

**`/pages`** emphasizes that Hyper is **hypermedia-first** - routes return HTML pages (or HTMX fragments), not JSON.

This structure:
- Keeps hypermedia endpoints (pages) separate from optional JSON endpoints (`/app/api`)
- Co-locates layouts and partials with the routes that use them
- Provides a clear application boundary (`/app` vs `/components`)
- Scales from simple apps to complex ones with business logic layers

### Why PascalCase for Non-Routes?

Using **PascalCase** (`Base.py`, `Form.py`) signals "this is a component-like thing, not a route":

1. **No import aliasing needed** - `from app.pages import Base` works directly
2. **Visual distinction** - At a glance, you know what's a route vs a layout/partial
3. **Semantic alignment** - Files used as components (`<{Base}>`) are named like components
4. **Simple router logic** - Skip PascalCase files when generating routes

Alternative conventions (underscores, reserved names) require more cognitive overhead or limit flexibility.

### Why Co-locate Partials with Routes?

Partials are **stateful, page-specific** template pieces. Keeping them next to their routes:

1. **Makes dependencies obvious** - No guessing which route uses which partial
2. **Reduces context switching** - Everything for a route is in one directory
3. **Enables clean imports** - Use relative imports: `from . import Form`
4. **Prevents premature abstraction** - Start in route, extract to partial only when needed

Only promote to `/components` after 3+ uses across different routes.

---

## Key Points

- **`/app/pages` = hypermedia endpoints (HTML)**
- **`/app/api` = JSON endpoints (optional)**
- **PascalCase = not a route (layouts, partials)**
- **`[param]` = path parameter**
- **`index.py` = directory route**
- **Type hints = automatic injection**
- **Directory routes = routes with partials**

---

**[← Previous: Overview](01-overview.md)** | **[Back to Index](README.md)** | **[Next: Templates →](03-templates.md)**
