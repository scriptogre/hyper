# Routing

Two approaches to define routes.

---

## Approach 1: File-Based Routing

File structure maps to URLs. Logic and template in the same file.

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

## File-Based: Dynamic Generation (SSG)

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

## File-Based: On-Demand Rendering (SSR)

No `generate()` needed. Renders per request.

```python
# app/pages/users/[id].py
id: int  # From URL

user = User.get(id)

t"""<h1>{user.name}</h1>"""
```

URL `/users/123` renders on each visit.

---

## Approach 2: Decorator-Based Routing

Define routes with decorators. Templates live separately.

Works like FastAPI.

```python
# app/main.py (or routes.py)
from hyper import Hyper
from templates import UserProfile, BlogPost

app = Hyper()

@app.get("/users/{id}")
def get_user(id: int):
    user = User.get(id)
    posts = user.get_posts()
    return UserProfile(user=user, posts=posts)

@app.get("/blog/{slug}")
def get_post(slug: str):
    post = Post.get_by_slug(slug)
    return BlogPost(post=post)
```

Templates are separate components:

```python
# templates/UserProfile.py
user: User
posts: list

t"""
<div>
  <h1>{user.name}</h1>
  <p>{len(posts)} posts</p>
</div>
"""
```

Clean separation. Explicit routing. Familiar FastAPI patterns.

---

## Which Approach?

**File-Based**
- Quick prototyping
- Simple sites
- Convention over configuration
- File system is the router

**Decorator-Based**
- Complex routing logic
- API endpoints
- Familiar FastAPI patterns
- Explicit route definitions

Choose what fits your project.

---

## Project Structure

### File-Based (SSG)

```
app/
  pages/          # Routes (file structure = URLs)
  content/        # Data (see content.md)

components/       # Shared UI
public/           # Static files
```

### File-Based (SSR)

```
app/
  pages/          # Routes (file structure = URLs)
  models/         # Database
  services/       # Business logic

components/       # Shared UI
```

### Decorator-Based

```
app/
  main.py         # Route definitions (or routes.py)
  models/         # Database
  services/       # Business logic

templates/        # Reusable components
```

---

## File-Based Routing Rules

- File = route
- `index.py` = directory path
- `[param]` = dynamic segment
- PascalCase = not a route
- Directories = nested paths
- `generate()` = SSG multiple pages
- No `generate()` = SSR on-demand

---

**[Next: Templates →](templates.md)**