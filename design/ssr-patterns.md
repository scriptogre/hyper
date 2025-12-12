# Advanced Features

> **🔮 SSR Mode** - Most features in this document require server-side rendering (planned). Some advanced patterns may work in SSG mode where applicable.

---

## HTMX Integration

Hyper is built for hypermedia-driven applications and works seamlessly with HTMX.

### Detecting HTMX Requests

```python
# routes/users/index.py
from hyper import Request
from routes._base import Layout

request: Request
users = get_all_users()

# Check if request is from HTMX
is_htmx = "HX-Request" in request.headers

if is_htmx:
    # Return partial HTML for HTMX swap
    t"""
    <div id="user-list">
        {[t'<div class="user">{u.name}</div>' for u in users]}
    </div>
    """
else:
    # Return full page
    t"""
    <{Layout} title="Users">
        <h1>Users</h1>
        <div id="user-list" hx-get="/users" hx-trigger="every 5s" hx-swap="innerHTML">
            {[t'<div class="user">{u.name}</div>' for u in users]}
        </div>
    </{Layout}>
    """
```

### HTMX Helpers

```python
from hyper import is_htmx, hx_redirect, hx_trigger, Request

request: Request

if is_htmx(request):
    # Helper function instead of manual header check
    t"""<div>Partial content</div>"""
else:
    t"""<html>Full page</html>"""
```

### Custom Response Headers

Set HTMX-specific headers:

```python
# routes/api/delete_user.py
from hyper import Request, Response

request: Request
response: Response
user_id: int

delete_user(user_id)

# Trigger client-side event
response.headers["HX-Trigger"] = "userDeleted"

# Or multiple events
response.headers["HX-Trigger"] = '{"userDeleted": {"id": ' + str(user_id) + '}}'

t"""<div>User deleted</div>"""
```

**See [Fragments](templates.md#fragments) for fragment-based HTMX patterns.**

---

## Response Manipulation

Inject `Response` to modify headers, status codes, and cookies:

### Setting Status Codes

```python
# routes/api/create_user.py
from hyper import Response

response: Response

user = create_user(...)

# Set status code
response.status_code = 201

t"""<div>User created: {user.id}</div>"""
```

### Custom Headers

```python
# routes/api/data.py
from hyper import Response

response: Response

# Set custom headers
response.headers["X-Custom-Header"] = "value"
response.headers["Cache-Control"] = "max-age=3600"

t"""<div>Data</div>"""
```

### Setting Cookies

```python
# routes/login.py
from hyper import POST, Response

response: Response

if POST:
    # Authenticate user
    user = authenticate(...)

    # Set session cookie
    response.set_cookie(
        key="session_id",
        value=user.session_id,
        httponly=True,
        secure=True,
        samesite="lax",
        max_age=86400  # 24 hours
    )

    t"""<div>Logged in!</div>"""
```

### Deleting Cookies

```python
# routes/logout.py
from hyper import Response

response: Response

# Delete cookie
response.delete_cookie("session_id")

t"""<div>Logged out</div>"""
```

---

## Multiple Return Types

Routes can return different response types:

### JSON Response

```python
# routes/api/users.py
from hyper import JSONResponse

users = get_all_users()

# Return JSON instead of HTML
JSONResponse([
    {"id": u.id, "name": u.name, "email": u.email}
    for u in users
])
```

### Plain Text Response

```python
# routes/health.py
from hyper import Response

Response(content="OK", media_type="text/plain")
```

### File Download

```python
# routes/download/export.py
from hyper import Response

csv_content = generate_csv_data()

Response(
    content=csv_content,
    media_type="text/csv",
    headers={
        "Content-Disposition": "attachment; filename=export.csv"
    }
)
```

---

## Redirects

```python
# routes/login.py
from hyper import GET, POST, RedirectResponse, Form
from typing import Annotated

if GET:
    # Show login form
    t"""<form method="POST">...</form>"""

elif POST:
    username: Annotated[str, Form()]
    password: Annotated[str, Form()]

    if authenticate(username, password):
        # Redirect to dashboard
        RedirectResponse(url="/dashboard", status_code=303)
    else:
        # Show error
        t"""
        <div class="error">Invalid credentials</div>
        <form method="POST">...</form>
        """
```

**Status codes:**
- `302` - Temporary redirect (default)
- `303` - See Other (recommended for POST → GET redirects)
- `307` - Temporary redirect (preserves method)
- `308` - Permanent redirect (preserves method)

---

## Static Files

Place static assets in the `static/` directory:

```
static/
  css/
    style.css
  js/
    app.js
  images/
    logo.png
```

Access them at `/static/*`:

```html
<link rel="stylesheet" href="/static/css/style.css">
<script src="/static/js/app.js"></script>
<img src="/static/images/logo.png" alt="Logo">
```

**Configuration:**

```python
# app.py
from hyper import Hyper

app = Hyper(
    static_dir="static",  # Default: "static"
    static_url="/static"  # Default: "/static"
)
```

---

## Middleware

Add Starlette middleware for additional functionality:

### Session Middleware

```python
# app.py
from hyper import Hyper
from starlette.middleware.sessions import SessionMiddleware

app = Hyper()

# Add session support
app.add_middleware(
    SessionMiddleware,
    secret_key="your-secret-key-here"
)
```

**Using sessions in routes:**

```python
# routes/counter.py
from hyper import Request

request: Request

# Get session
session = request.session

# Increment counter
count = session.get("count", 0) + 1
session["count"] = count

t"""<html><body><h1>Count: {count}</h1></body></html>"""
```

### CORS Middleware

```python
# app.py
from hyper import Hyper
from starlette.middleware.cors import CORSMiddleware

app = Hyper()

# Add CORS
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
# app.py
from hyper import Hyper
from starlette.middleware.base import BaseHTTPMiddleware

class CustomMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        # Before request
        print(f"Request: {request.url}")

        response = await call_next(request)

        # After request
        print(f"Response: {response.status_code}")

        return response

app = Hyper()
app.add_middleware(CustomMiddleware)
```

---

## Custom Error Pages

```python
# app.py
from hyper import Hyper
from starlette.responses import HTMLResponse

app = Hyper()

@app.exception_handler(404)
async def not_found(request, exc):
    return HTMLResponse(
        """
        <!doctype html>
        <html>
        <head><title>404 Not Found</title></head>
        <body>
            <h1>Page Not Found</h1>
            <p>The page you're looking for doesn't exist.</p>
            <a href="/">Go Home</a>
        </body>
        </html>
        """,
        status_code=404
    )

@app.exception_handler(500)
async def server_error(request, exc):
    return HTMLResponse(
        """
        <!doctype html>
        <html>
        <head><title>500 Server Error</title></head>
        <body>
            <h1>Server Error</h1>
            <p>Something went wrong. Please try again later.</p>
        </body>
        </html>
        """,
        status_code=500
    )
```

---

## Lifecycle Events

Add startup and shutdown handlers:

```python
# app.py
from hyper import Hyper

app = Hyper()

@app.on_event("startup")
async def startup():
    print("Starting up...")
    # Initialize database, cache, etc.
    await connect_to_database()

@app.on_event("shutdown")
async def shutdown():
    print("Shutting down...")
    # Close connections
    await close_database()
```

---

## WebSocket Support

Hyper inherits WebSocket support from Starlette:

```python
# routes/ws/chat.py (requires special setup)
from starlette.websockets import WebSocket

async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()

    while True:
        data = await websocket.receive_text()
        await websocket.send_text(f"Echo: {data}")
```

**Note:** WebSocket routing requires additional configuration. See Starlette's [WebSocket documentation](https://www.starlette.io/websockets/).

---

## Application Configuration

```python
# app.py
from hyper import Hyper

app = Hyper(
    pages_dir="pages",        # Default: "pages"
    static_dir="static",      # Default: "static"
    debug=True,               # Default: False
)
```

**Options:**
- `pages_dir` - Directory containing route files
- `static_dir` - Directory for static files
- `debug` - Enable debug mode (detailed error pages)

---

## Key Points

- **HTMX detection via request headers**
- **Inject `Response` to modify headers/status/cookies**
- **Multiple response types (JSON, text, files)**
- **Redirects with `RedirectResponse`**
- **Static files served from `static/` directory**
- **Middleware for sessions, CORS, and custom logic**
- **Custom error handlers with `@app.exception_handler`**
- **Lifecycle events for startup/shutdown**

---

**[← Previous: Streaming](streaming.md)**
