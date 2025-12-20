# Simple Real-Time Proposal: HTMX + SSE + _hyperscript

**Principle:** Use the simplest tool for the job. No magic.

---

## The Stack

### Level 1: HTMX (Client → Server)

For 95% of interactions:

```python
# app/pages/search.py

query = request.params.get("q", "")
results = search(query) if query else []

t"""
<input
    name="q"
    hx-get="/search"
    hx-trigger="keyup changed delay:300ms"
    hx-target="#results"
/>

<div id="results">
    {% for item in results %}
    <div>{item}</div>
    {% endfor %}
</div>
"""
```

**What it solves:**
- Forms, search, CRUD
- 99% of web app interactions
- Simple, debuggable, works everywhere

---

### Level 2: SSE (Server → Client)

For server-initiated updates:

```python
# app/sse/notifications.py

from hyper import sse

@sse
async def notifications():
    """Server-sent events stream"""
    user_id = current_user.id

    async for notification in subscribe_to_notifications(user_id):
        yield t"""
        <div
            class="notification"
            hx-swap-oob="beforeend:#notifications"
        >
            {notification.message}
        </div>
        """
```

**Client:**
```html
<div
    id="notifications"
    hx-ext="sse"
    sse-connect="/sse/notifications"
    sse-swap="message"
>
</div>
```

**What it solves:**
- Real-time notifications
- Live updates from server
- Dashboard updates
- Progress indicators

**Why SSE over WebSocket:**
- ✅ Simpler (one-way: server → client)
- ✅ Standard HTTP (works through proxies)
- ✅ Auto-reconnects
- ✅ Built-in browser support
- ✅ Works with HTMX perfectly

---

### Level 3: _hyperscript (Client-Side)

For UI interactions:

```python
# app/components/dropdown.py

items: list[str]

t"""
<div class="dropdown">
    <button _="on click toggle .open on closest .dropdown">
        Select ▾
    </button>

    <ul class="dropdown-menu">
        {% for item in items %}
        <li _="on click
            put my innerText into previous <button/> then
            remove .open from closest .dropdown
        ">
            {item}
        </li>
        {% endfor %}
    </ul>
</div>
"""
```

**What it solves:**
- Dropdowns, modals, accordions
- Animations, transitions
- Client-side state (no server needed)

---

### Level 4: WebSocket (Explicit, Rare)

For truly bidirectional real-time (chat, collaboration):

```python
# app/websockets/chat.py

from hyper import WebSocket
import json

async def chat(ws: WebSocket, room: str):
    """Explicit WebSocket endpoint"""
    await ws.accept()

    # Join room
    await pubsub.subscribe(f"chat:{room}")

    try:
        # Handle incoming messages
        async for msg in ws.receive_text():
            data = json.loads(msg)

            # Broadcast to room
            await pubsub.publish(f"chat:{room}", {
                "user": current_user.name,
                "text": data["text"]
            })

        # Handle published messages
        async for msg in pubsub.listen():
            html = t"""
            <div class="message">
                <strong>{msg.user}:</strong> {msg.text}
            </div>
            """
            await ws.send_text(html)

    finally:
        await pubsub.unsubscribe(f"chat:{room}")
        await ws.close()
```

**Client:**
```html
<div id="chat">
    <div id="messages"></div>

    <form _="
        on submit
            make a WebSocket to /ws/chat?room=general called ws
            on message from ws put the event.data before end of #messages
            on submit send { text: #input.value } to ws then set #input.value to ''
    ">
        <input id="input" />
    </form>
</div>
```

**What it solves:**
- Real-time chat
- Collaborative editing
- Multiplayer games

**Why explicit:**
- ✅ Full control
- ✅ No magic
- ✅ Clear what's happening
- ✅ Easy to debug

---

## Complete Example: Real-Time Todo App

### The Component

```python
# app/pages/todos.py

from hyper import Request, sse, POST

request: Request
todos = db.get_todos(user_id=current_user.id)

# Handle HTMX requests
if POST:
    if request.headers.get("HX-Trigger") == "add-todo":
        text = request.form["text"]
        todo = db.create_todo(user_id=current_user.id, text=text)

        # Notify via SSE
        notify(current_user.id, "todo_added", todo)

        # Return new todo HTML
        return t"""
        <li hx-swap-oob="beforeend:#todo-list">
            <input type="checkbox" />
            <span>{todo.text}</span>
        </li>
        """

# Main template
t"""
<div>
    <!-- Form: uses HTMX -->
    <form
        hx-post="/todos"
        hx-trigger="submit"
        hx-target="#todo-list"
        hx-swap="beforeend"
        _="on htmx:afterRequest set #text.value to ''"
    >
        <input id="text" name="text" />
        <button type="submit">Add</button>
    </form>

    <!-- Todo list -->
    <ul id="todo-list">
        {% for todo in todos %}
        <li>
            <input
                type="checkbox"
                checked={todo.done}
                hx-post="/todos/{todo.id}/toggle"
                hx-swap="outerHTML"
            />
            <span>{todo.text}</span>
        </li>
        {% endfor %}
    </ul>

    <!-- SSE connection for real-time updates -->
    <div
        hx-ext="sse"
        sse-connect="/sse/todos"
        sse-swap="message"
    ></div>
</div>
"""
```

### SSE Endpoint

```python
# app/sse/todos.py

from hyper import sse

@sse
async def todos():
    """Real-time todo updates"""
    user_id = current_user.id

    async for event in subscribe_to_events(user_id):
        if event.type == "todo_added":
            yield t"""
            <li hx-swap-oob="beforeend:#todo-list">
                <input type="checkbox" />
                <span>{event.data.text}</span>
            </li>
            """

        elif event.type == "todo_completed":
            yield t"""
            <li hx-swap-oob="outerHTML:[data-todo-id='{event.data.id}']">
                <input type="checkbox" checked />
                <span class="done">{event.data.text}</span>
            </li>
            """
```

**What we have:**
- ✅ HTMX for user actions (add, toggle)
- ✅ SSE for real-time updates (when others add todos)
- ✅ _hyperscript for UI (clear input after submit)
- ✅ No magic, no WebSocket, no state binding

---

## Hyper Framework Support

### 1. Make HTMX Easy

```python
# Built-in HTMX helpers

from hyper import hx

# Auto-detect HTMX requests
if hx.request:
    # Return just the fragment
    return t"""<div>Updated content</div>"""
else:
    # Return full page
    return t"""
    <html>
        <body>
            <div>Updated content</div>
        </body>
    </html>
    """

# Helper for triggering events
hx.trigger("todoAdded", after="swap")

# Helper for redirects
hx.redirect("/todos")

# Helper for retargeting
hx.retarget("#new-target")
```

---

### 2. Make SSE Simple

```python
from hyper import sse

@sse
async def updates():
    """Simple SSE endpoint"""
    async for update in subscribe():
        yield t"<div>{update}</div>"

# Or with events
@sse
async def updates():
    async for update in subscribe():
        yield ("custom-event", t"<div>{update}</div>")

# Or with IDs (for reconnection)
@sse
async def updates():
    id = 1
    async for update in subscribe():
        yield (id, t"<div>{update}</div>")
        id += 1
```

**Framework handles:**
- ✅ Proper headers (`text/event-stream`)
- ✅ Keep-alive
- ✅ Error handling
- ✅ Connection cleanup

---

### 3. Document WebSocket Pattern

Don't build it into the framework. Just document the pattern:

```python
# app/websockets/chat.py

from hyper import WebSocket

async def chat(ws: WebSocket):
    await ws.accept()

    async for message in ws.receive_text():
        # Handle message
        html = process(message)
        await ws.send_text(html)
```

**Provide utilities, not magic:**

```python
# hyper/websocket.py

class WebSocketManager:
    """Optional utility for managing connections"""

    def __init__(self):
        self.connections = {}

    async def connect(self, room: str, ws: WebSocket):
        if room not in self.connections:
            self.connections[room] = set()
        self.connections[room].add(ws)

    async def broadcast(self, room: str, message: str):
        for ws in self.connections.get(room, []):
            await ws.send_text(message)

# Use it if you want, or roll your own
```

---

## What We Don't Build

### ❌ Live State Binding
```python
# NO: magic state sync
count = 0
def increment():
    global count
    count += 1
```

### ❌ Auto WebSocket
```python
# NO: directory-based WebSocket
app/live/counter.py  # Automatically stateful
```

### ❌ `{}` Compilation
```python
# NO: special syntax
<button _="on click {increment}">
```

### ❌ Stateful Components
```python
# NO: component state management
@live
def component(): pass
```

---

## What We DO Build

### ✅ HTMX Helpers

```python
from hyper import hx

if hx.request:
    return fragment
else:
    return full_page
```

### ✅ SSE Decorator

```python
@sse
async def updates():
    yield t"<div>Update</div>"
```

### ✅ WebSocket Utilities

```python
from hyper import WebSocket

async def chat(ws: WebSocket):
    # You have full control
    pass
```

### ✅ _hyperscript Integration

Make it easy to use _hyperscript in templates:

```python
t"""
<button _="on click toggle .active">
    Toggle
</button>
"""
```

---

## Developer Experience

### Simple Case (95%)

```python
# Just HTMX
t"""
<form hx-post="/submit">
    <input name="email" />
    <button>Submit</button>
</form>
"""
```

### Real-Time Case (4%)

```python
# HTMX + SSE
t"""
<div hx-ext="sse" sse-connect="/updates" sse-swap="message">
    Content updates here
</div>
"""

@sse
async def updates():
    async for update in subscribe():
        yield t"<div>{update}</div>"
```

### Complex Case (1%)

```python
# Explicit WebSocket
async def chat(ws: WebSocket):
    # You write the logic
    # No framework magic
    pass
```

---

## Benefits of This Approach

### 1. **Graduated Complexity**
- Start with HTMX (simple)
- Add SSE if needed (still simple)
- Drop to WebSocket if truly needed (explicit)

### 2. **No Magic**
- Everything is visible
- Easy to debug
- Standard protocols

### 3. **Easy to Understand**
- HTMX: client → server (HTTP)
- SSE: server → client (HTTP)
- WebSocket: bidirectional (explicit)

### 4. **Plays Well Together**
```html
<!-- Mix HTMX + SSE + _hyperscript -->
<div
    hx-get="/data"
    hx-trigger="every 2s"
    hx-ext="sse"
    sse-connect="/updates"
    _="on sse:message add .highlight then wait 1s then remove .highlight"
>
    Content
</div>
```

### 5. **Escape Hatches**
- Need more control? Drop to lower level
- Want less abstraction? Use raw HTTP/WebSocket
- Framework doesn't lock you in

---

## Comparison

| Feature | Live State (Proposed) | HTMX + SSE (This) |
|---------|----------------------|-------------------|
| **Complexity** | High | Low |
| **Magic** | Lots | None |
| **Debuggable** | Hard | Easy |
| **Learning curve** | Steep | Gentle |
| **Mental models** | 2 (HTMX + Live) | 1 (HTTP) |
| **Code to maintain** | Lots | Little |
| **Real-time** | Yes | Yes (SSE) |
| **Bidirectional** | Yes (WebSocket) | Yes (explicit WS) |
| **Works for 95%** | Yes | Yes |

---

## Recommendation

**Build HTMX + SSE support. Document WebSocket patterns. Don't build live state binding.**

### Why?

1. **Simpler**: Less code, less magic, less to learn
2. **Hyper-like**: Simple things simple, complex things possible
3. **Debuggable**: Standard HTTP, visible in network tab
4. **Flexible**: Compose HTMX + SSE + _hyperscript as needed
5. **Maintainable**: Less framework code to maintain

### What to Build

**Phase 1: HTMX helpers**
```python
from hyper import hx

if hx.request:
    # Helper functions for HTMX responses
    pass
```

**Phase 2: SSE support**
```python
@sse
async def updates():
    yield t"<div>Update</div>"
```

**Phase 3: WebSocket docs**
- Tutorial on WebSocket with Hyper
- Example patterns (chat, notifications)
- No framework magic, just show how

**Phase 4: _hyperscript docs**
- How to use with HTMX
- Common patterns
- Examples

---

## The Hard Truth

Live state binding is:
- ❌ Months of work
- ❌ High complexity
- ❌ Lots of magic
- ❌ Hard to debug
- ❌ Solves rare problems

HTMX + SSE is:
- ✅ Days of work
- ✅ Low complexity
- ✅ No magic
- ✅ Easy to debug
- ✅ Solves 99% of problems

**Let's build the simple thing.**

---

*The best framework is the one you don't notice.*
