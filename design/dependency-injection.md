# Dependency Injection

Hyper uses type hints to inject values into your routes. The injection mechanism differs between SSG and SSR modes.

---

## ✓ SSG - Path Parameters

In SSG mode, path parameters are injected at build time using `Literal` type hints.

### Static Values

```python
# app/pages/docs/[lang].py
from typing import Literal

# Path parameter (injected by CLI at build time)
lang: Literal['en', 'es', 'fr']

t"""
<html>
<body>
    <h1>Documentation - {lang}</h1>
    <p>Language: {lang}</p>
</body>
</html>
"""
```

**Result:** Generates `/docs/en`, `/docs/es`, `/docs/fr`

### Dynamic Values from Content

```python
# app/pages/blog/[slug].py
from typing import Literal
from app.content import blogs

# Path parameter (injected by CLI)
slug: Literal[*[b.slug for b in blogs]]

# ---

# Use injected value
post = next(b for b in blogs if b.slug == slug)

t"""
<html>
<body>
    <h1>{post.title}</h1>
    <article>{post.html}</article>
</body>
</html>
"""
```

**Result:** Generates one page per blog post

**The `# ---` separator** distinguishes injected variables (above) from computed variables (below).

See [ssg.md](ssg.md) for full SSG path generation patterns.

---

## 🔮 SSR - Path Parameters (Planned)

Variables with type hints and **no default value** become path parameters:

```python
# routes/users/{user_id}.py
from models import User

user_id: int  # Automatically injected from URL path

user = User.get(id=user_id)

t"""
<html>
<body>
    <h1>Profile: {user.name}</h1>
    <p>User ID: {user_id}</p>
</body>
</html>
"""
```

**URL:** `/users/123`
**Result:** `user_id` is injected as `123` (converted to int)

**Type conversion:**
- `int` - Converted to integer
- `str` - No conversion
- `float` - Converted to float
- Custom types - See FastAPI's [Path Parameters](https://fastapi.tiangolo.com/tutorial/path-params/) docs

---

## 🔮 SSR - Query Parameters (Planned)

Variables with type hints and **a default value** become query parameters:

```python
# routes/search.py

q: str = ""          # ?q=...
limit: int = 10      # ?limit=...
page: int = 1        # ?page=...

results = search(q, limit=limit, page=page)

t"""
<html>
<body>
    <h1>Search Results for "{q}"</h1>
    <p>Found {len(results)} results</p>
</body>
</html>
"""
```

**URL Examples:**
- `/search` → `q=""`, `limit=10`, `page=1` (defaults)
- `/search?q=python&limit=20` → `q="python"`, `limit=20`, `page=1`

**See FastAPI's [Query Parameters](https://fastapi.tiangolo.com/tutorial/query-params/) docs for more patterns.**

---

## 🔮 SSR - Query Parameter Validation (Planned)

Use `Annotated` with `Query()` for validation:

```python
# routes/search.py
from typing import Annotated
from hyper import Query

# With validation
q: Annotated[str, Query(min_length=1, max_length=100)] = ""
limit: Annotated[int, Query(ge=1, le=100)] = 10
page: Annotated[int, Query(ge=1)] = 1

results = search(q, limit=limit, page=page)

t"""<html><body>Results: {len(results)}</body></html>"""
```

**Validation options:**
- `min_length`, `max_length` - String length
- `ge`, `le`, `gt`, `lt` - Numeric constraints
- `regex` - Pattern matching
- `deprecated` - Mark as deprecated

**See FastAPI's [Query Parameters and String Validations](https://fastapi.tiangolo.com/tutorial/query-params-str-validations/) docs.**

---

## 🔮 SSR - Headers (Planned)

Inject HTTP headers with `Header()`:

```python
# routes/profile.py
from typing import Annotated
from hyper import Header

# Automatically converts user_agent → "User-Agent"
user_agent: Annotated[str, Header()] = "Unknown"

# Custom header name
is_htmx: Annotated[bool, Header(alias="HX-Request")] = False

t"""
<html>
<body>
    <h1>Your Profile</h1>
    <p>User Agent: {user_agent}</p>
    <p>HTMX Request: {is_htmx}</p>
</body>
</html>
"""
```

**See FastAPI's [Header Parameters](https://fastapi.tiangolo.com/tutorial/header-params/) docs.**

---

## 🔮 SSR - Cookies (Planned)

Inject cookies with `Cookie()`:

```python
# routes/dashboard.py
from typing import Annotated
from hyper import Cookie

session_id: Annotated[str | None, Cookie()] = None

if session_id:
    user = get_user_from_session(session_id)
    t"""<html><body><h1>Welcome, {user.name}!</h1></body></html>"""
else:
    t"""<html><body><h1>Please log in</h1></body></html>"""
```

**See FastAPI's [Cookie Parameters](https://fastapi.tiangolo.com/tutorial/cookie-params/) docs.**

---

## 🔮 SSR - Request Body (JSON) (Planned)

Use Pydantic models for JSON request bodies:

```python
# routes/api/users.py
from typing import Annotated
from pydantic import BaseModel, EmailStr
from hyper import Body

class UserCreate(BaseModel):
    name: str
    email: EmailStr
    age: int | None = None

# Inject parsed and validated body
user_data: Annotated[UserCreate, Body()]

# Create user with validated data
new_user = create_user(
    name=user_data.name,
    email=user_data.email,
    age=user_data.age
)

t"""<html><body><h1>Created user {new_user.id}</h1></body></html>"""
```

**See FastAPI's [Request Body](https://fastapi.tiangolo.com/tutorial/body/) and [Body - Fields](https://fastapi.tiangolo.com/tutorial/body-fields/) docs.**

---

## 🔮 SSR - Form Data (Planned)

Use `Form()` to extract form fields:

```python
# routes/contact.py
from typing import Annotated
from hyper import GET, POST, Form

if GET:
    # Show form
    t"""
    <html>
    <body>
        <form method="POST">
            <input name="name" required>
            <input name="email" type="email" required>
            <textarea name="message" required></textarea>
            <button>Submit</button>
        </form>
    </body>
    </html>
    """

elif POST:
    # Form fields automatically injected
    name: Annotated[str, Form()]
    email: Annotated[str, Form()]
    message: Annotated[str, Form()]

    send_contact_email(name, email, message)

    t"""<html><body><h1>Thank you, {name}!</h1></body></html>"""
```

**See [forms.md](forms.md) for full form handling patterns and FastAPI's [Form Data](https://fastapi.tiangolo.com/tutorial/request-forms/) docs.**

---

## 🔮 SSR - Form Models (Planned)

Use Pydantic models for structured form data:

```python
# routes/signup.py
from typing import Annotated
from pydantic import BaseModel, EmailStr
from hyper import GET, POST, Form

class SignupForm(BaseModel):
    username: str
    email: EmailStr
    password: str
    age: int

if GET:
    # Show signup form
    t"""<html><body><form method="POST">...</form></body></html>"""

elif POST:
    # Submit form and show welcome
    form: Annotated[SignupForm, Form()]

    user = create_user(
        username=form.username,
        email=form.email,
        password=hash_password(form.password),
        age=form.age
    )

    t"""<html><body><h1>Welcome, {user.username}!</h1></body></html>"""
```

**See FastAPI's [Form Models](https://fastapi.tiangolo.com/tutorial/request-form-models/) docs.**

---

## 🔮 SSR - File Uploads (Planned)

Handle file uploads with `File()` and `UploadFile`:

```python
# routes/upload.py
from typing import Annotated
from hyper import Request, File, UploadFile, GET, POST

if GET:
    t"""
    <html>
    <body>
        <form method="POST" enctype="multipart/form-data">
            <input type="file" name="file">
            <button>Upload</button>
        </form>
    </body>
    </html>
    """

elif POST:
    # Single file
    file: Annotated[UploadFile, File()]

    contents = await file.read()
    save_file(file.filename, contents)

    t"""<html><body><h1>Uploaded {file.filename}</h1></body></html>"""
```

Note: Use `GET` or `POST` helpers for concise syntax.

**Multiple files:**

```python
# routes/upload-multiple.py
from typing import Annotated
from hyper import Request, File, UploadFile, POST

if POST:
    # Multiple files
    files: Annotated[list[UploadFile], File()]

    for file in files:
        contents = await file.read()
        save_file(file.filename, contents)

    t"""<html><body><h1>Uploaded {len(files)} files</h1></body></html>"""
```

**See FastAPI's [Request Files](https://fastapi.tiangolo.com/tutorial/request-files/) docs.**

---

## 🔮 SSR - Request Object (Planned)

Inject the full Starlette `Request` object:

```python
# routes/debug.py
from hyper import Request

request: Request

t"""
<html>
<body>
    <h1>Request Info</h1>
    <p>Method: {request.method}</p>
    <p>URL: {request.url}</p>
    <p>Path: {request.url.path}</p>
    <p>Client: {request.client.host}</p>
</body>
</html>
"""
```

---

## 🔮 SSR - Response Manipulation (Planned)

Inject `Response` to set headers, status codes, and cookies:

```python
# routes/set-cookie.py
from hyper import Response

response: Response

# Set a cookie
response.set_cookie(key="session_id", value="abc123", httponly=True)

# Set custom headers
response.headers["X-Custom-Header"] = "value"

# Set status code
response.status_code = 201

t"""<html><body><h1>Cookie set!</h1></body></html>"""
```

**See [ssr-patterns.md](ssr-patterns.md) for more response manipulation patterns.**

---

## 🔮 SSR - Combining Multiple Injections (Planned)

You can combine different injection types:

```python
# routes/api/posts/{post_id}.py
from typing import Annotated
from pydantic import BaseModel
from hyper import Request, Header, Body, PUT

class PostUpdate(BaseModel):
    title: str
    content: str

# Inject path param
post_id: int

# Inject request
request: Request

# Inject header
auth_token: Annotated[str, Header(alias="Authorization")]

if PUT:
    # Inject JSON body
    post_data: Annotated[PostUpdate, Body()]

    # Verify auth
    user = verify_token(auth_token)

    # Update post
    post = update_post(
        post_id=post_id,
        user_id=user.id,
        title=post_data.title,
        content=post_data.content
    )

    t"""<html><body><h1>Updated: {post.title}</h1></body></html>"""
```

---

## Key Differences from FastAPI

| Feature | FastAPI | Hyper |
|---------|---------|--------|
| **Scope** | Function parameters | Module-level variables |
| **When injected** | During function call | Before module execution |
| **Where usable** | Inside function only | Entire module + template |
| **Syntax** | `def route(param: Type)` | `param: Type` at module level |

**Why module-level?**
- Variables available throughout the module
- No function wrapper needed
- Direct use in templates
- Simpler for route handlers

---

## Advanced Patterns

For more advanced patterns, see FastAPI's comprehensive docs:

- **[Path Parameters and Numeric Validations](https://fastapi.tiangolo.com/tutorial/path-params-numeric-validations/)**
- **[Query Parameter Models](https://fastapi.tiangolo.com/tutorial/query-param-models/)**
- **[Body - Multiple Parameters](https://fastapi.tiangolo.com/tutorial/body-multiple-params/)**
- **[Body - Nested Models](https://fastapi.tiangolo.com/tutorial/body-nested-models/)**

The patterns translate directly - just use module-level type hints instead of function parameters.

---

## Key Points

- **Type hints = automatic injection**
- **No default = path parameter**
- **With default = query parameter**
- **Use `Annotated` for advanced config**
- **Module-level scope (not function parameters)**
- **Available everywhere in module + template**
- **Most FastAPI patterns translate directly**

---

**[← Previous: Content](content.md)** | **[Next: Forms →](forms.md)**
