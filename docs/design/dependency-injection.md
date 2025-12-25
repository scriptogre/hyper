# Dependency Injection

Type hints inject values into your pages.

---

## Pages Are Like Functions

A page file works like a FastAPI route function.

**FastAPI:**
```python
@app.get("/users/{id}")
def get_user(id: int, request: Request):
    return {"user": id}
```

**Hyper:**
```python
# app/pages/users/[id].py
id: int
request: Request

t"""<h1>User {id}</h1>"""
```

Module-level type hints = function parameters.

---

## Path Parameters

Match `[param]` in filename to type hint in file.

```python
# app/pages/users/[id].py
id: int  # From URL path

user = get_user(id)
```

**URL:** `/users/123` → `id = 123`

Type conversion automatic:
- `int` → integer
- `str` → string
- `float` → float

---

## Query Parameters

Type hints with defaults become query parameters.

```python
# app/pages/search.py
q: str = ""
limit: int = 10
page: int = 1

results = search(q, limit, page)
```

**URLs:**
- `/search` → `q=""`, `limit=10`, `page=1`
- `/search?q=python` → `q="python"`, `limit=10`, `page=1`
- `/search?q=python&limit=20` → `q="python"`, `limit=20`, `page=1`

---

## Request Object

Inject the full request.

```python
# app/pages/debug.py
from hyper import Request

request: Request

t"""
<h1>Request Info</h1>
<p>Method: {request.method}</p>
<p>URL: {request.url}</p>
<p>Path: {request.url.path}</p>
"""
```

---

## Response Object

Inject response to set headers, cookies, status.

```python
# app/pages/api/data.py
from hyper import Response

response: Response

# Set headers
response.headers["X-Custom"] = "value"

# Set status
response.status_code = 201

t"""<h1>Created</h1>"""
```

---

## Headers

Inject specific headers.

```python
from hyper import Header
from typing import Annotated

user_agent: Annotated[str, Header()] = "Unknown"
is_htmx: Annotated[bool, Header(alias="HX-Request")] = False

t"""
<p>User Agent: {user_agent}</p>
<p>HTMX: {is_htmx}</p>
"""
```

---

## Cookies

Inject cookies.

```python
from hyper import Cookie
from typing import Annotated

session_id: Annotated[str | None, Cookie()] = None

if session_id:
    user = get_user(session_id)
```

---

## Form Data

Inject form fields.

```python
from hyper import Form, GET, POST
from typing import Annotated

if GET:
    t"""
    <form method="POST">
        <input name="name" required>
        <input name="email" type="email" required>
        <button>Submit</button>
    </form>
    """

if POST:
    name: Annotated[str, Form()]
    email: Annotated[str, Form()]

    save_contact(name, email)

    t"""<h1>Thank you, {name}!</h1>"""
```

Use Pydantic models for complex forms:

```python
from pydantic import BaseModel

class ContactForm(BaseModel):
    name: str
    email: str
    message: str

if POST:
    form: Annotated[ContactForm, Form()]

    save_contact(form.name, form.email, form.message)
```

---

## JSON Body

Inject parsed JSON.

```python
from hyper import Body
from pydantic import BaseModel
from typing import Annotated

class UserCreate(BaseModel):
    name: str
    email: str

user_data: Annotated[UserCreate, Body()]

user = create_user(user_data.name, user_data.email)
```

---

## File Uploads

Inject uploaded files.

```python
from hyper import File, UploadFile, POST
from typing import Annotated

if POST:
    file: Annotated[UploadFile, File()]

    contents = await file.read()
    save_file(file.filename, contents)

    t"""<h1>Uploaded {file.filename}</h1>"""
```

---

## SSG Mode

SSG uses `generate()` for dynamic routes.

```python
# app/pages/blog/[slug].py
from app.content import blogs

def generate():
    for blog in blogs:
        yield {"slug": blog.slug, "blog": blog}

# Injected at build time
slug: str
blog: Blog

# ---

t"""<h1>{blog.title}</h1>"""
```

See [ssg.md](ssg.md) for details.

---

## Multiple Injections

Combine different types.

```python
# app/pages/api/posts/[id].py
from hyper import Request, Header, Body, PUT
from pydantic import BaseModel
from typing import Annotated

class PostUpdate(BaseModel):
    title: str
    content: str

# Path parameter
id: int

# Request
request: Request

# Header
auth: Annotated[str, Header(alias="Authorization")]

if PUT:
    # JSON body
    data: Annotated[PostUpdate, Body()]

    user = verify_token(auth)
    post = update_post(id, user.id, data.title, data.content)

    t"""<h1>Updated: {post.title}</h1>"""
```

---

## Key Differences from FastAPI

| Feature | FastAPI | Hyper |
|---------|---------|-------|
| Scope | Function parameters | Module-level variables |
| When injected | During function call | Before module execution |
| Where usable | Inside function only | Entire module + template |

Module-level means:
- Variables available everywhere
- No function wrapper needed
- Direct use in templates

---

## Rules Summary

- Type hint = injection
- No default = path parameter
- With default = query parameter
- `Annotated[T, X()]` = advanced config
- Module-level scope
- Available in template

---

**[← Previous: Routing](routing.md)** | **[Next: SSG →](ssg.md)**
