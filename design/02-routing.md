# Routing

Hyper uses **file-based routing** - your file structure automatically maps to URLs.

---

## Project Structure

```
routes/          # Pages and API endpoints
  index.py              → /
  about.py              → /about
  contact.py            → /contact
  users/
    index.py            → /users
    {user_id}.py        → /users/{user_id}
    {user_id}/
      edit.py           → /users/{user_id}/edit
      posts/
        {post_id}.py    → /users/{user_id}/posts/{post_id}
  api/
    posts.py            → /api/posts

layouts/         # Layout templates (not routes)
  Base.py               # PascalCase for layouts
  Blog.py
  Admin.py

components/      # Reusable components (not routes)
  UserCard.py           # PascalCase for components
  Button.py
  Alert.py
```

---

## Rules

### `{param}` Creates Path Parameters

```
routes/
  users/
    {user_id}.py      → /users/123
    {user_id}/
      posts/
        {post_id}.py  → /users/123/posts/456
```

Parameters are injected via type hints (see [05-dependency-injection.md](05-dependency-injection.md)).

### `index.py` Maps to Directory Path

```
routes/
  index.py          → /
  users/
    index.py        → /users
  blog/
    index.py        → /blog
```

### Nested Directories Create Nested Paths

```
routes/
  api/
    v1/
      users.py      → /api/v1/users
```

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
# routes/users/{user_id}.py
from app.models import User

user_id: int  # Automatically injected from URL path

user = User.get(id=user_id)

t"""
<!doctype html>
<html>
<head><title>{user.name}</title></head>
<body>
    <h1>Profile: {user.name}</h1>
    <p>Email: {user.email}</p>
    <p>User ID: {user_id}</p>
</body>
</html>
"""
```

**URL:** `/users/123`
**Result:** `user_id` is injected as `123` (converted to int)

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
# routes/contact.py
from hyper import GET, POST
from layouts import Base

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

## Key Points

- **`routes/` = URL structure**
- **`layouts/` = layout templates (not routes)**
- **`components/` = reusable components (not routes)**
- **`{param}` = path parameter**
- **`index.py` = directory route**
- **Type hints = automatic injection**

---

**[← Previous: Overview](01-overview.md)** | **[Back to Index](README.md)** | **[Next: Templates →](03-templates.md)**
