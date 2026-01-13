# Server-Side Rendering

> **Status**: 🚧 In development - Starting with framework integration

Hyper templates render server-side with any Python framework.

---

## Basic Usage

### With FastAPI

```python
from fastapi import FastAPI
from fastapi.responses import HTMLResponse
from pages import Home

# Set default response class
app = FastAPI(default_response_class=HTMLResponse)

@app.get("/")
def index():
    return Home(title="Welcome")
```

Or per-route:

```python
@app.get("/", response_class=HTMLResponse)
def index():
    return Home(title="Welcome")
```

### With Django

```python
from django.http import HttpResponse
from pages import Home

def index(request):
    return HttpResponse(Home(title="Welcome"))
```

### With Flask

```python
from flask import Flask
from pages import Home

app = Flask(__name__)

@app.route("/")
def index():
    return Home(title="Welcome")
```

Templates return strings. Any framework works.

---

## Simplified Syntax (Planned)

> **Status**: 🔮 Compiler optimization planned

The compiler will auto-import common types and transform simplified syntax.

**You write**:
```hyper
request: Request
response: Response
email: str | None = Form()
session_id: str | None = Cookie()
```

**Compiler generates**:
```python
from hyper import Request, Response, Form, Cookie
from typing import Annotated

def Template(
    request: Request,
    response: Response,
    email: Annotated[str | None, Form()] = None,
    session_id: Annotated[str | None, Cookie()] = None
) -> str:
    ...
```

**Auto-imports**: Compiler detects `Request`, `Form()`, `Cookie()` and adds imports.

**Syntax transform**: `str = Form()` becomes `Annotated[str, Form()]` in generated code.

Your `.hyper` file stays clean. Generated Python uses best practices. IDE support works via compiled version.

---

## File-Based Routing (Planned)

> **Status**: 🔮 In design

Map file structure to URLs automatically.

**File structure**:
```
pages/
├── index.hyper          → /
├── about.hyper          → /about
└── blog/
    ├── index.hyper      → /blog
    └── [slug].hyper     → /blog/:slug
```

**Dynamic route** (`pages/blog/[slug].hyper`):

```hyper
from models import Post

slug: str  # Prop from URL

---

post = Post.get(slug=slug)

<!doctype html>
<html>
<head>
    <title>{post.title}</title>
</head>
<body>
    <article>
        <h1>{post.title}</h1>
        <div>{safe(post.html)}</div>
    </article>
</body>
</html>
```

**Header (above `---`)**: Props, imports, definitions
**Body (below `---`)**: Local variables, HTML

The `slug` prop comes from the URL. Query string params also become props.

---

## Request and Response (Planned)

> **Status**: 🔮 In design

Inject request and response objects via type hints.

```hyper
from hyper import Request, Response

request: Request
response: Response

---

user_agent = request.headers.get("user-agent")
is_htmx = "HX-Request" in request.headers

response.headers["X-Custom"] = "value"
response.status_code = 201

<div>
    <p>User agent: {user_agent}</p>
    if is_htmx:
        <p>HTMX request detected</p>
    end
</div>
```

---

## Forms

### Simple Form

```hyper
from hyper import Request, Form
from typing import Annotated

request: Request
email: Annotated[str | None, Form()] = None

---

if request.method == "GET":
    <form method="POST">
        <input name="email" type="email" required />
        <button>Subscribe</button>
    </form>

elif request.method == "POST":
    save_subscriber(email)
    <p>Thanks for subscribing!</p>
```

Form props are optional. FastAPI extracts them if form data exists.

### With Validation

```hyper
from pydantic import BaseModel, EmailStr
from hyper import Request, Form
from typing import Annotated

class ContactForm(BaseModel):
    name: str
    email: EmailStr
    message: str

request: Request
form: Annotated[ContactForm | None, Form()] = None

---

if request.method == "GET":
    <form method="POST">
        <input name="name" required />
        <input name="email" type="email" required />
        <textarea name="message" required></textarea>
        <button>Send</button>
    </form>

elif request.method == "POST":
    send_email(form.name, form.email, form.message)
    <p>Message sent!</p>
```

---

## HTMX Integration

### Partial vs Full Page

```hyper
from hyper import Request
from components import Layout

request: Request

---

users = get_all_users()
is_htmx = "HX-Request" in request.headers

if is_htmx:
    # Return partial
    <div id="user-list">
        for user in users:
            <div class="user">{user.name}</div>
        end
    </div>

elif not is_htmx:
    # Return full page
    <{Layout} title="Users">
        <h1>Users</h1>
        <div
            id="user-list"
            hx-get="/users"
            hx-trigger="every 5s"
            hx-swap="innerHTML"
        >
            for user in users:
                <div class="user">{user.name}</div>
            end
        </div>
    </{Layout}>
```

### Setting Headers

```hyper
from hyper import Request, Response, Form
from typing import Annotated

request: Request
response: Response
user_id: Annotated[int | None, Form()] = None

---

if request.method == "POST":
    delete_user(user_id)
    response.headers["HX-Trigger"] = "userDeleted"
    <div>User deleted</div>
```

### With Fragments (Planned)

```hyper
users: list[User]

---

<div class="page">
    fragment UserList:
        for user in users:
            <div class="user">{user.name}</div>
        end
    end
</div>
```

Import fragments separately for HTMX endpoints:

```python
from pages.Users import UserList

@app.get("/users/list", response_class=HTMLResponse)
def users_list():
    return UserList(users=get_all_users())
```

---

## Streaming (Planned)

> **Status**: 🔮 Exploring design

Stream responses incrementally.

```python
from fastapi.responses import StreamingResponse

@app.get("/feed")
def feed():
    return StreamingResponse(
        Feed(posts=all_posts),
        media_type="text/html"
    )
```

The template automatically yields chunks:

```hyper
posts: list[Post]

---

<div class="feed">
    for post in posts:
        <article>
            <h1>{post.title}</h1>
            <p>{post.excerpt}</p>
        </article>
    end
</div>
```

Each loop iteration yields. No syntax changes needed.

### Server-Sent Events

```python
@app.get("/notifications")
async def notifications():
    async def stream():
        async for notification in subscribe_notifications():
            yield f"event: notification\n"
            yield f"data: {notification.html}\n\n"

    return StreamingResponse(
        stream(),
        media_type="text/event-stream"
    )
```

Client connects:

```html
<div
    hx-ext="sse"
    sse-connect="/notifications"
    sse-swap="notification"
    hx-swap="beforeend"
>
    <!-- Notifications appear here -->
</div>
```

---

## Background Tasks (Planned)

> **Status**: 🔮 In design

Queue tasks to run after response.

```hyper
from hyper import Request, BackgroundTasks, Form
from typing import Annotated

request: Request
background_tasks: BackgroundTasks
email: Annotated[str | None, Form()] = None
message: Annotated[str | None, Form()] = None

---

if request.method == "POST":
    background_tasks.add_task(send_email, email, message)
    <p>Message queued!</p>
```

Response returns immediately. Email sends in background.

### Async Tasks

```hyper
from hyper import Request, BackgroundTasks
import httpx

request: Request
background_tasks: BackgroundTasks

async def send_webhook(url: str, data: dict):
    async with httpx.AsyncClient() as client:
        await client.post(url, json=data)

---

if request.method == "POST":
    background_tasks.add_task(send_webhook, webhook_url, payload)
    <p>Done!</p>
```

---

## Cookies and Sessions

### Setting Cookies

```hyper
from hyper import Request, Response, Form
from typing import Annotated

request: Request
response: Response
username: Annotated[str | None, Form()] = None
password: Annotated[str | None, Form()] = None

---

if request.method == "POST":
    user = authenticate(username, password)

    response.set_cookie(
        key="session_id",
        value=user.session_id,
        httponly=True,
        secure=True,
        max_age=86400
    )

    <p>Logged in!</p>
```

### Reading Cookies

```hyper
from hyper import Cookie
from typing import Annotated

session_id: Annotated[str | None, Cookie()] = None

---

if session_id:
    user = get_user(session_id)
    <p>Welcome back, {user.name}!</p>

elif not session_id:
    <p>Please log in</p>
```

### Sessions with Middleware

Configure session middleware:

```python
from starlette.middleware.sessions import SessionMiddleware

app.add_middleware(
    SessionMiddleware,
    secret_key="your-secret-key"
)
```

Use in templates:

```hyper
from hyper import Request

request: Request

---

session = request.session
count = session.get("count", 0) + 1
session["count"] = count

<p>Count: {count}</p>
```

---

## Redirects

```hyper
from hyper import Request, RedirectResponse, Form
from typing import Annotated

request: Request
username: Annotated[str | None, Form()] = None
password: Annotated[str | None, Form()] = None

---

if request.method == "GET":
    <form method="POST">
        <input name="username" required />
        <input name="password" type="password" required />
        <button>Login</button>
    </form>

elif request.method == "POST":
    if authenticate(username, password):
        RedirectResponse(url="/dashboard", status_code=303)

    elif not authenticate(username, password):
        <div class="error">Invalid credentials</div>
        <form method="POST">
            <input name="username" required />
            <input name="password" type="password" required />
            <button>Login</button>
        </form>
```

Status codes:
- `302` - Temporary (default)
- `303` - See Other (use for POST → GET)
- `307` - Temporary (preserves method)
- `308` - Permanent (preserves method)

---

## File Uploads

```hyper
from hyper import Request, File, UploadFile
from typing import Annotated

request: Request
file: Annotated[UploadFile | None, File()] = None

---

if request.method == "POST":
    contents = await file.read()
    save_file(file.filename, contents)
    <p>Uploaded {file.filename}</p>
```

---

## Error Handling

### Custom Error Pages

```python
from pages import NotFound

app = FastAPI(default_response_class=HTMLResponse)

@app.exception_handler(404)
async def not_found(request, exc):
    return NotFound(path=request.url.path)
```

**Template** (`pages/NotFound.hyper`):

```hyper
path: str

---

<!doctype html>
<html>
<head>
    <title>404 Not Found</title>
</head>
<body>
    <h1>Page Not Found</h1>
    <p>The page <code>{path}</code> doesn't exist.</p>
    <a href="/">Go Home</a>
</body>
</html>
```

---

## Middleware

### CORS

```python
from starlette.middleware.cors import CORSMiddleware

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

### Custom Middleware

```python
from starlette.middleware.base import BaseHTTPMiddleware
import time

class TimingMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        start = time.time()
        response = await call_next(request)
        duration = time.time() - start
        response.headers["X-Process-Time"] = str(duration)
        return response

app.add_middleware(TimingMiddleware)
```

---

## Static Files

Configure static file serving:

```python
app = Hyper(
    static_dir="static",
    static_url="/static"
)
```

Use in templates:

```hyper
<!doctype html>
<html>
<head>
    <link rel="stylesheet" href="/static/css/style.css" />
    <script src="/static/js/app.js"></script>
</head>
<body>
    <img src="/static/images/logo.png" alt="Logo" />
</body>
</html>
```

---

**See Also**:
- [Templates](templates.md) - Template syntax
- [SSG](ssg.md) - Static site generation
