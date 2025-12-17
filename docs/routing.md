# Routing

File structure maps to URLs.

---

## Basic Routes

Create a file. Get a route.

```
app/pages/
  index.py        → /
  about.py        → /about
  contact.py      → /contact
```

---

## Directory Routes

Use `index.py` for the directory path.

```
app/pages/
  blog/
    index.py      → /blog
```

---

## Nested Routes

Directories create nested paths.

```
app/pages/
  api/
    v1/
      users.py    → /api/v1/users
```

---

## Dynamic Routes

Use `[param]` for path parameters.

```
app/pages/
  users/
    [id].py       → /users/123
  blog/
    [slug].py     → /blog/hello-world
```

Parameter injected as variable:

```python
# app/pages/users/[id].py
id: int  # Injected from URL

# Use it
user = get_user(id)
```

See [dependency-injection.md](dependency-injection.md) for details.

---

## Multiple Parameters

Nest parameters in directories.

```
app/pages/
  [lang]/
    blog/
      [slug].py   → /en/blog/hello
                  → /es/blog/hola
```

Both parameters injected:

```python
# app/pages/[lang]/blog/[slug].py
lang: str
slug: str

# ...
```

---

## SSG: Dynamic Generation

Define `generate()` to create multiple pages.

```python
# app/pages/blog/[slug].py

def generate():
    yield {"slug": "intro"}
    yield {"slug": "advanced"}

slug: str  # Injected

# ---

t"""<h1>Post: {slug}</h1>"""
```

Generates:
- `/blog/intro/index.html`
- `/blog/advanced/index.html`

See [ssg.md](ssg.md) for details.

---

## SSR: On-Demand Rendering

No `generate()` needed. Renders per request.

```python
# app/pages/users/[id].py
id: int  # From URL

user = User.get(id)

t"""<h1>{user.name}</h1>"""
```

URL `/users/123` renders on each visit.

---

## Project Structure

### SSG

```
app/
  pages/          # Routes
  content/        # Data (see content.md)

components/       # Shared UI
public/           # Static files
```

### SSR

```
app/
  pages/          # Routes
  api/            # JSON endpoints (optional)
  models/         # Database
  services/       # Business logic

components/       # Shared UI
```

---

## Rules Summary

- File = route
- `index.py` = directory path
- `[param]` = dynamic segment
- PascalCase = not a route
- Directories = nested paths
- `generate()` = SSG multiple pages
- No `generate()` = SSR on-demand

---

**[Next: Templates →](templates.md)**