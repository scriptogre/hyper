# Hyper Live V3: Two-Way Binding with _hyperscript

## Core Philosophy

**Client-side state (UI toggles, dropdowns) → _hyperscript**
**Server-side state (data, validation, persistence) → Python**

Don't send "dropdown expanded" to the server. That's wasteful.
DO send "save user" to the server. That needs persistence.

---

## Design Principles

1. **_hyperscript is first-class** for client-side interactivity
2. **Curly brackets `{}`** already mean "Python/server" in t-strings
3. **Extend that pattern** to event handlers
4. **Two-way binding**: Client → Server AND Server → Client

---

## Proposed Syntax

### Client-Side (Pure _hyperscript)

```python
t"""
<button _="on click toggle .hidden on #dropdown">
    Toggle
</button>

<div id="dropdown" class="hidden">
    Dropdown content
</div>
"""
```

**No server involved.** Pure client-side DOM manipulation.

---

### Server-Side (Curly Brackets)

```python
# app/live/counter.py

count = 0

def increment():
    global count
    count += 1

t"""
<div>
    <p>Count: {count}</p>
    <button _="on click {increment}">+</button>
</div>
"""
```

**`{increment}` signals:** This calls Python on the server.

---

### Mixed (Client + Server)

```python
# app/live/form.py

def save(name: str):
    # Save to database
    db.save(name)

t"""
<form>
    <input name="name" />

    <button _="
        on click
            add .loading to me then
            {save(name)} then
            remove .loading from me
    ">
        Save
    </button>
</form>
"""
```

**Flow:**
1. Add loading class (client-side)
2. Call `{save(name)}` on server (waits for response)
3. Remove loading class (client-side)

---

## How It Works

### Syntax Rules

**Inside `_` attribute:**
- Regular text = _hyperscript (runs on client)
- `{expression}` = Python call (runs on server)

**Examples:**

```html
<!-- Pure client -->
<button _="on click toggle .active">Toggle</button>

<!-- Pure server -->
<button _="on click {increment}">+</button>

<!-- Mixed -->
<button _="
    on click
        add .loading
        {increment}
        remove .loading
">
    +
</button>

<!-- Server with parameters -->
<button _="on click {delete(123)}">Delete</button>

<!-- From form inputs -->
<form>
    <input name="email" />
    <button _="on click {subscribe(email)}">Subscribe</button>
</form>
```

---

## Two-Way Binding

### Client → Server

Client triggers server update via `{handler}`:

```python
# app/live/todos.py

todos = []

def add_todo(text: str):
    todos.append(text)

t"""
<form>
    <input name="text" />
    <button _="on click {add_todo(text)}">Add</button>
</form>
"""
```

---

### Server → Client

Server pushes updates when state changes:

```python
# app/live/dashboard.py
from hyper import shared, broadcast

metrics = shared({"users": 0, "revenue": 0})

def refresh():
    metrics.update(fetch_latest_metrics())
    broadcast()  # Pushes update to all connected clients

t"""
<div>
    <h2>Users: {metrics.users}</h2>
    <h2>Revenue: ${metrics.revenue}</h2>

    <button _="on click {refresh}">Refresh</button>
</div>
"""
```

**When `broadcast()` is called:**
1. Server re-renders component
2. Computes HTML diff
3. Sends diff via WebSocket
4. Client morphs DOM

---

## Real-World Examples

### Example 1: Expandable Card (Client-Only)

```python
# app/components/card.py

title: str

t"""
<div class="card">
    <h3 _="on click toggle .expanded on closest .card">
        {title} ▾
    </h3>
    <div class="content">
        {...}
    </div>
</div>

<style>
.card .content { display: none; }
.card.expanded .content { display: block; }
</style>
"""
```

**No server state.** Pure _hyperscript.

---

### Example 2: Live Search (Server State)

```python
# app/live/search.py
import asyncio

query = ""
results = []

async def search(q: str):
    global query, results
    query = q
    results = await db.search(q)

t"""
<div>
    <input
        name="query"
        value="{query}"
        _="on keyup debounced at 300ms {search(value)}"
    />

    <ul>
        {% for item in results %}
        <li>{item}</li>
        {% endfor %}
    </ul>
</div>
"""
```

**Server state:** query and results must be on server (database access).

---

### Example 3: Todo List (Mixed)

```python
# app/live/todos.py

todos = []

def add_todo(text: str):
    global todos
    if text.strip():
        todos.append({"text": text, "done": False})

def toggle(index: int):
    global todos
    todos[index]["done"] = not todos[index]["done"]

def delete(index: int):
    global todos
    todos.pop(index)

t"""
<div>
    <form>
        <input name="text" />
        <button _="
            on click
                {add_todo(text)} then
                set #text.value to ''
        ">
            Add
        </button>
    </form>

    <ul>
        {% for i, todo in enumerate(todos) %}
        <li>
            <!-- Server: toggle state -->
            <input
                type="checkbox"
                checked={todo.done}
                _="on change {toggle({i})}"
            />

            <!-- Client: strikethrough -->
            <span _="
                on change from previous <input/>
                    toggle .done
            ">
                {todo.text}
            </span>

            <!-- Mixed: fade out, then delete -->
            <button _="
                on click
                    add .fade-out to closest <li/> then
                    wait 300ms then
                    {delete({i})}
            ">×</button>
        </li>
        {% endfor %}
    </ul>
</div>
"""
```

**Flow:**
1. Check box → Server updates `done` state → Re-render
2. Client listens to checkbox change → Adds `.done` class (instant feedback)
3. Delete button → Fade out (client) → Call server → Re-render

---

### Example 4: Chat (Shared State + Server Push)

```python
# app/live/chat.py
from hyper import shared, broadcast

messages = shared([])
username: str  # From session

def send(text: str):
    if text.strip():
        messages.append({"user": username, "text": text})
        broadcast()  # Push to all clients

t"""
<div>
    <div id="messages">
        {% for msg in messages %}
        <div class="message">
            <strong>{msg.user}:</strong> {msg.text}
        </div>
        {% endfor %}
    </div>

    <form>
        <input name="text" />
        <button _="
            on click
                {send(text)} then
                set #text.value to ''
        ">
            Send
        </button>
    </form>
</div>
"""
```

**Server push:** When anyone sends a message, ALL connected clients receive update via WebSocket.

---

### Example 5: Multi-Step Form (Client State + Server Validation)

```python
# app/live/signup.py

errors = {}
submitted = False

def validate(email: str, password: str):
    global errors, submitted
    errors = {}

    if "@" not in email:
        errors["email"] = "Invalid email"
    if len(password) < 8:
        errors["password"] = "Password too short"

    if not errors:
        save_user(email, password)
        submitted = True

t"""
<div>
    {% if submitted %}
    <div class="success">Account created!</div>
    {% else %}

    <!-- Client-side: multi-step wizard -->
    <div id="step1" class="step">
        <input name="email" />
        <button _="on click hide #step1 then show #step2">
            Next
        </button>
    </div>

    <div id="step2" class="step" style="display: none">
        <input name="password" type="password" />
        <button _="
            on click
                add .loading then
                {validate(email, password)}
        ">
            Submit
        </button>

        {% if errors %}
        <div class="errors">
            {% for field, msg in errors.items() %}
            <p>{msg}</p>
            {% endfor %}
        </div>
        {% endif %}
    </div>

    {% endif %}
</div>
"""
```

**Step navigation = client**
**Validation = server**

---

## Implementation: How `{handler}` Works

### 1. Template Compilation

When Hyper compiles the template, it detects `{}` inside `_` attributes:

```python
# Before
<button _="on click {increment}">+</button>

# After (compiled)
<button
    _="on click send increment to server"
    data-live-component="counter-abc123"
>+</button>
```

**Transformation:**
- `{increment}` → `send increment to server`
- Adds `data-live-component` for WebSocket connection

---

### 2. Client-Side _hyperscript Extension

Add a custom _hyperscript command `send ... to server`:

```javascript
// hyper-live-hyperscript.js

_hyperscript.addCommand("send", function(parser, runtime, tokens) {
    // Parse: send <handler> to server
    const handler = parser.requireElement("expression", tokens);
    parser.requireToken("to", tokens);
    parser.requireToken("server", tokens);

    return {
        execute: async function(ctx) {
            const componentEl = ctx.me.closest('[data-live-component]');
            const componentId = componentEl.dataset.liveComponent;

            // Send to server via WebSocket
            const result = await window.hyperLive.call(
                componentId,
                handler,
                extractParams(ctx)
            );

            return result;
        }
    };
});
```

---

### 3. WebSocket Communication

```javascript
// hyper-live.js

class HyperLive {
    async call(componentId, handler, params) {
        // Send to server
        const messageId = generateId();

        this.ws.send(JSON.stringify({
            type: "call",
            id: messageId,
            component: componentId,
            handler: handler,
            params: params
        }));

        // Wait for response
        return new Promise((resolve) => {
            this.pending[messageId] = resolve;
        });
    }

    onMessage(message) {
        if (message.type === "result") {
            // Handler finished, resolve promise
            this.pending[message.id](message.result);
        } else if (message.type === "update") {
            // Server pushed an update
            this.morphDOM(message.component, message.html);
        }
    }
}
```

---

### 4. Server-Side Handler

```python
# hyper/live/handler.py

async def handle_call(connection, message):
    component_id = message["component"]
    handler_name = message["handler"]
    params = message["params"]

    # Get component instance
    component = get_component(component_id)

    # Execute handler
    handler = getattr(component, handler_name)
    result = await handler(**params)

    # Send result back
    await connection.send_json({
        "type": "result",
        "id": message["id"],
        "result": result
    })

    # Re-render and send update
    html = component.render()
    await connection.send_json({
        "type": "update",
        "component": component_id,
        "html": html
    })
```

---

## The _hyperscript Integration

### Built-in Commands

Add Hyper-specific _hyperscript commands:

**`send <handler> to server`**
```html
<button _="on click send increment to server">+</button>
```

**`debounced at <ms>`**
```html
<input _="on keyup debounced at 300ms send search(value) to server" />
```

**`with loading on <selector>`**
```html
<button _="
    on click
        send save to server with loading on me
">
    Save
</button>
```

---

### Syntactic Sugar

The `{}` syntax is sugar for the _hyperscript command:

```html
<!-- Sugar -->
<button _="on click {increment}">+</button>

<!-- Desugars to -->
<button _="on click send increment to server">+</button>
```

**Both work!** Use `{}` for brevity, use explicit `send...to server` for clarity.

---

## Configuration

### Mark Components as "Live"

Same as V2 - directory convention:

```
app/
├── components/     # Static (no server state)
├── live/          # Live (server state + WebSocket)
└── pages/
```

OR module-level marker:

```python
# app/components/counter.py

live = True  # Enable WebSocket for this component

count = 0
```

---

## Connection Management

### Auto-Connect

When component with `data-live-component` is rendered:
1. Client establishes WebSocket connection
2. Server creates isolated namespace for this connection
3. Component ready for two-way binding

### Lifecycle

```python
def on_mount():
    """Called when WebSocket connects"""
    print(f"User connected")

def on_unmount():
    """Called when WebSocket disconnects"""
    print(f"User disconnected")
```

---

## Benefits of This Design

### 1. **Clear Separation**

```html
<!-- Client-side (no {}) -->
<button _="on click toggle .active">Toggle</button>

<!-- Server-side (with {}) -->
<button _="on click {save}">Save</button>
```

Visual distinction: `{}` = server call.

### 2. **Progressive Enhancement**

```html
<form action="/subscribe" method="POST">
    <input name="email" />

    <button _="on click {subscribe(email)}">
        Subscribe
    </button>
</form>
```

**With JS:** WebSocket call
**Without JS:** Form POST

### 3. **Composable**

```html
<button _="
    on click
        add .loading to me then
        {save} then
        remove .loading from me then
        add .success to me then
        wait 2s then
        remove .success from me
">
    Save
</button>
```

Mix client and server logic seamlessly.

### 4. **Leverages _hyperscript Ecosystem**

All _hyperscript features work:
- `debounced`, `throttled`
- `toggle`, `add`, `remove`
- `send`, `trigger`
- `wait`, `repeat`
- Full expressions

### 5. **Familiar Syntax**

`{}` already means "Python" in Hyper. Same pattern here.

---

## Comparison

| Aspect | Pure _hyperscript | `{}` Integration |
|--------|-------------------|------------------|
| **Client toggle** | `_="on click toggle .active"` | Same |
| **Server call** | `_="on click send increment to server"` | `_="on click {increment}"` |
| **Debounce** | `_="on keyup debounced send search to server"` | `_="on keyup debounced {search(value)}"` |
| **Mixed** | Verbose | Clean with `{}` |

---

## Open Questions

### 1. Parameter Extraction

How to extract form values in `{handler(param)}`?

**Option A: Auto-extract from form**
```html
<form>
    <input name="email" />
    <button _="on click {subscribe(email)}">
        <!-- email auto-extracted from form -->
    </button>
</form>
```

**Option B: Explicit `value`**
```html
<input name="email" _="on input {validate(value)}" />
```

**Recommendation:** Both! Auto-extract from form, use `value` for single inputs.

---

### 2. Return Values

Should `{handler}` return a value usable in _hyperscript?

```html
<button _="
    on click
        set result to {increment} then
        put result into #display
">
    +
</button>
```

**Recommendation:** Yes! Return values enable richer client logic.

---

### 3. Error Handling

What if `{handler}` throws?

```python
def save():
    raise ValueError("Database error")
```

**Option A: Return error in response**
```javascript
const result = await send(handler);
if (result.error) {
    // Show error
}
```

**Option B: Trigger _hyperscript event**
```html
<div _="on error from server show #error-message">
    ...
</div>
```

**Recommendation:** Option B (more _hyperscript-y)

---

## Summary

**The API:**

1. **Use _hyperscript** for client-side interactivity
2. **Use `{handler}`** inside `_` for server calls
3. **Put component in `app/live/`** to enable WebSocket
4. **Use `broadcast()`** for server-push updates

**Example:**

```python
# app/live/counter.py

count = 0

def increment():
    global count
    count += 1

t"""
<div>
    <p>Count: {count}</p>

    <!-- Client-side animation -->
    <button _="
        on click
            add .pulse to me then
            {increment} then
            remove .pulse from me
    ">
        +
    </button>
</div>
"""
```

**This design:**
- ✅ Makes _hyperscript first-class
- ✅ Uses `{}` for server (consistent with t-strings)
- ✅ Enables two-way binding
- ✅ Clear client vs server distinction
- ✅ Composable and powerful

**The minimal syntax: just add `{}` around server calls.**
