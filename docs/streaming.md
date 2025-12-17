# Streaming & Server-Sent Events

> **🔮 SSR Mode** - This feature requires server-side rendering (planned). Streaming and SSE require a running server and are not available in SSG mode.

Hyper provides first-class support for streaming responses and Server-Sent Events (SSE), perfect for real-time updates with HTMX.

---

## Async/Await Support

Hyper supports async operations while maintaining 100% legal Python syntax that works with all editors and tooling.

### Sync Routes (No Async Needed)

```python
# routes/about.py
from models import Company

company = Company.get(...)

t"""
<html>
<body>
    <h1>About {company.name}</h1>
</body>
</html>
"""
```

### Async Routes

When you need `await`, wrap your async logic in an async function:

```python
# routes/users/{user_id}.py
from models import User, Post

user_id: int  # Inject from URL params

user: User
posts: list[Post]

async def load():
    global user, posts
    user = await User.get(id=user_id)
    posts = await Post.get(user=user)

# Template uses the loaded data
t"""
<html>
<body>
    <h1>{user.name}</h1>
    <ul>
        {[t'<li>{p.title}</li>' for p in posts]}
    </ul>
</body>
</html>
"""
```

**Framework behavior:**
1. Detects if there's an async function in the module
2. If found, awaits it before continuing
3. Then finds and renders the template

**Important:**
- **100% Legal Python** - Works with all linters, formatters, and type checkers
- **Flexible Naming** - Name your async function anything you want
- **Optional** - Only add async function when you need it
- **Global Scope** - Use `global` keyword to make variables available in template
- **Single Function** - Framework calls the first async function it finds

---

## Server-Sent Events (SSE)

If your route has an **async generator function** (uses `yield`), the framework automatically creates a streaming response.

### Yield Formats

#### 1. T-String Only (Unnamed HTML Event)

```python
yield t"""<div>HTML content</div>"""
```

Converted to SSE format:
```
data: <div>HTML content</div>

```

#### 2. Tuple: (event_name, content)

```python
yield ("eventName", t"""<div>HTML</div>""")
yield ("eventName", {"key": "value"})
```

Converted to SSE format:
```
event: eventName
data: <div>HTML</div>

```

#### 3. Dictionary: {"event": ..., "data": ...}

```python
yield {
    "event": "update",
    "data": {"count": 42}
}
```

Converted to SSE format:
```
event: update
data: {"count": 42}

```

---

## Real-World Examples

### Live Notifications

```python
# routes/notifications/stream.py
from models import Notification
from services import subscribe_to_notifications

user_id: int

async def stream():
    # 1. String format (unnamed events)
    yield t"""<div class="info">Connected to notifications</div>"""

    async for notification in subscribe_to_notifications(user_id):
        # 2. Tuple format
        yield (
            "notification",
            t"""
            <div class="notification">
                <strong>{notification.title}</strong>
                <p>{notification.message}</p>
                <time>{notification.timestamp}</time>
            </div>
        """)

        # 3. Dictionary format
        yield {
            "event": "count",
            "data": {"unread": notification.unread_count}
        }
```

### Real-Time Chat

```python
# routes/chat/{room_id}/stream.py
import asyncio
import datetime
from services.chat import subscribe_to_room

room_id: str

async def stream():
    # Send connection event
    yield {
        "event": "connected",
        "data": {"room": room_id, "time": str(datetime.now())}
    }

    # Stream messages
    async for message in subscribe_to_room(room_id):
        # HTML message for HTMX to insert
        yield (
            "message",
            t"""
        <div class="message" id="msg-{message.id}">
            <img src="{message.user.avatar}" alt="{message.user.name}">
            <div>
                <strong>{message.user.name}</strong>
                <p>{message.text}</p>
                <time>{message.timestamp}</time>
            </div>
        </div>
        """)
```

---

## HTMX Integration

### Basic SSE Connection

```python
# routes/live-feed.py
from routes._base import Layout

t"""
<{Layout}>
    <h1>Live Feed</h1>
    <div
        hx-ext="sse"
        sse-connect="/stream/feed"
        sse-swap="message"
        hx-swap="beforeend"
    >
        <!-- Messages appear here -->
    </div>
</{Layout}>
"""
```

```python
# routes/stream/feed.py
import asyncio

async def stream():
    # Initial message
    yield ("message", t"""<div class="info">Feed connected</div>""")

    # Stream updates
    while True:
        await asyncio.sleep(2)

        yield ("message", t"""
        <div class="post">
            <h3>New Post at {datetime.now()}</h3>
            <p>Some exciting content!</p>
        </div>
        """)
```

### Multiple Event Types

```python
# routes/dashboard-live.py
from routes._base import Layout

t"""
<{Layout}>
    <h1>Dashboard</h1>

    <div id="metrics"
         hx-ext="sse"
         sse-connect="/stream/metrics"
    >
        <div sse-swap="metric" hx-swap="innerHTML">
            <!-- Metrics update here -->
        </div>

        <div sse-swap="alert" hx-swap="beforeend">
            <!-- Alerts appear here -->
        </div>

        <div id="status" sse-swap="status">
            <!-- Status updates here -->
        </div>
    </div>
</{Layout}>
"""
```

```python
# routes/stream/metrics.py
import asyncio

async def stream():
    while True:
        # Get current metrics
        metrics = await get_system_metrics()

        # Update metrics display
        yield ("metric", t"""
        <div class="metrics">
            <span>CPU: {metrics.cpu}%</span>
            <span>Memory: {metrics.memory}%</span>
            <span>Requests: {metrics.requests}/s</span>
        </div>
        """)

        # Send alert if needed
        if metrics.cpu > 80:
            yield ("alert", t"""
            <div class="alert warning">
                High CPU usage detected: {metrics.cpu}%
            </div>
            """)

        await asyncio.sleep(1)
```

---

## HTML Streaming (Progressive Rendering)

For large pages, stream HTML chunks as they're ready:

```python
# routes/report/{report_id}.py
from models import Report

report_id: int

async def stream():
    # Send page header immediately
    yield t"""
    <html>
    <head><title>Report {report_id}</title></head>
    <body>
        <h1>Report {report_id}</h1>
    """

    # Stream sections as they're generated
    summary = await generate_summary(report_id)
    yield t"""
    <section id="summary">
        <h2>Summary</h2>
        <div>{summary}</div>
    </section>
    """

    charts = await generate_charts(report_id)
    yield t"""
    <section id="charts">
        <h2>Charts</h2>
        <div>{charts}</div>
    </section>
    """

    # Close page
    yield t"""
    </body>
    </html>
    """
```

---

## Streaming with Fragments

Combine fragments with streaming for powerful patterns. See [Streaming with Fragments](templates.md#streaming-with-fragments) for details.

```python
from hyper import render, fragment

user_id: int
user = None

def load_data():
    global user
    user = get_user(user_id)

t"""
<html>
<body>
    <div id="status" {fragment}>
        Status: {user.status}
    </div>
</body>
</html>
"""

async def stream():
    load_data()
    yield render(fragments="status")

    async for update in subscribe_to_updates(user_id):
        load_data()  # Re-fetch
        yield render(fragments="status")  # Fresh data!
```

---

## Custom Response Control

For full control over streaming responses:

```python
# routes/download/csv.py
from hyper import StreamingResponse

async def generate_csv():
    yield "name,email,created_at\n"

    async for user in User.stream():
        yield f"{user.name},{user.email},{user.created_at}\n"

# Return explicit StreamingResponse
response = StreamingResponse(
    generate_csv(),
    media_type="text/csv",
    headers={
        "Content-Disposition": "attachment; filename=users.csv"
    }
)
```

---

## Key Points

- **Async function = awaited before rendering**
- **Async generator (with `yield`) = streaming response**
- **Three yield formats: t-string, tuple, dict**
- **Perfect for HTMX SSE extension**
- **Combine with fragments for re-runnable logic**
- **Progressive HTML rendering for large pages**
- **100% legal Python syntax**

---

**[← Previous: Forms](forms.md)** | **[Next: Static Site Generation →](ssg.md)**
