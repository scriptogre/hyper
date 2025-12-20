# Hyper Live: Minimalist Server State Binding (V2)

## Core Principle: Keep the Top-Level t-string

Hyper components are module-level type hints + top-level t-string. Let's keep that for live components too.

---

## Design: Convention-Based Live Components

### Option A: Directory Convention (Recommended)

**Just put the component in `app/live/` directory:**

```python
# app/live/counter.py

# State: module-level variables
count = 0

# Handlers: module-level functions
def increment():
    global count
    count += 1

def decrement():
    global count
    count -= 1

# Template: top-level t-string (unchanged!)
t"""
<div class="counter">
    <h2>Count: {count}</h2>
    <button @click="decrement">−</button>
    <button @click="increment">+</button>
</div>
"""
```

**That's it!** Files in `app/live/` are automatically stateful.

---

### Option B: Module-Level Marker

**Add a single line at the top:**

```python
# app/components/counter.py
live = True  # This line makes it stateful

count = 0

def increment():
    global count
    count += 1

t"""
<div>
    <p>{count}</p>
    <button @click="increment">+</button>
</div>
"""
```

---

### Option C: Special Import

**Import triggers live mode:**

```python
# app/components/counter.py
from hyper import live  # Presence of this import = live component

count = 0

def increment():
    global count
    count += 1

t"""
<div>
    <p>{count}</p>
    <button @click="increment">+</button>
</div>
"""
```

---

## Recommendation: **Option A (Directory)**

Why?
1. ✅ **Zero boilerplate** - not even an import
2. ✅ **Clear separation** - live vs static components
3. ✅ **Scales well** - easy to see what's stateful
4. ✅ **Most Hyper-like** - convention over configuration

---

## Complete Examples

### Counter

```python
# app/live/counter.py

count = 0

def increment():
    global count
    count += 1

def decrement():
    global count
    count -= 1

def reset():
    global count
    count = 0

t"""
<div class="counter">
    <h2>Count: {count}</h2>
    <div class="controls">
        <button @click="decrement">−</button>
        <button @click="reset">Reset</button>
        <button @click="increment">+</button>
    </div>
</div>
"""
```

### Todo List

```python
# app/live/todos.py

todos = []
next_id = 1

def add_todo(text: str):
    global next_id
    if not text.strip():
        return

    todos.append({
        "id": next_id,
        "text": text.strip(),
        "done": False
    })
    next_id += 1

def toggle(todo_id: int):
    for todo in todos:
        if todo["id"] == todo_id:
            todo["done"] = not todo["done"]
            break

def delete(todo_id: int):
    global todos
    todos = [t for t in todos if t["id"] != todo_id]

# Computed value
remaining = len([t for t in todos if not t["done"]])

t"""
<div class="todo-app">
    <h1>Tasks <small>({remaining} remaining)</small></h1>

    <form @submit.prevent="add_todo(text)">
        <input name="text" placeholder="What needs to be done?" autofocus />
        <button type="submit">Add</button>
    </form>

    <ul class="todo-list">
        {% for todo in todos %}
        <li class="{'done' if todo.done else ''}">
            <input
                type="checkbox"
                checked={todo.done}
                @change="toggle({todo.id})"
            />
            <span>{todo.text}</span>
            <button class="delete" @click="delete({todo.id})">×</button>
        </li>
        {% endfor %}
    </ul>

    {% if todos %}
    <footer>
        <button @click="clear_completed">Clear completed</button>
    </footer>
    {% endif %}
</div>
"""
```

### Search

```python
# app/live/search.py
import asyncio
from app.data import search_items

query = ""
results = []
loading = False

async def search(q: str):
    global query, results, loading

    query = q
    if not q:
        results = []
        return

    loading = True
    await render()  # Force render to show loading state

    results = await search_items(q)
    loading = False

t"""
<div class="search-box">
    <input
        type="search"
        placeholder="Search..."
        value="{query}"
        @input.debounce.300="search(value)"
    />

    {% if loading %}
    <div class="loading">Searching...</div>
    {% elif query and not results %}
    <div class="empty">No results for "{query}"</div>
    {% elif results %}
    <ul class="results">
        {% for item in results %}
        <li>{item}</li>
        {% endfor %}
    </ul>
    {% endif %}
</div>
"""
```

### Chat (Shared State)

```python
# app/live/chat.py
from hyper import shared, broadcast

# Shared across ALL connections
messages = shared([])

# Per-connection state
username: str  # Injected from session/auth

def on_mount():
    """Called when user connects"""
    messages.append({
        "type": "system",
        "text": f"{username} joined"
    })
    broadcast()

def on_unmount():
    """Called when user disconnects"""
    messages.append({
        "type": "system",
        "text": f"{username} left"
    })
    broadcast()

def send_message(text: str):
    if not text.strip():
        return

    messages.append({
        "type": "message",
        "user": username,
        "text": text.strip()
    })
    broadcast()

t"""
<div class="chat-room">
    <div class="messages">
        {% for msg in messages %}
        {% if msg.type == "system" %}
        <div class="system-message">{msg.text}</div>
        {% else %}
        <div class="message">
            <strong>{msg.user}:</strong> {msg.text}
        </div>
        {% endif %}
        {% endfor %}
    </div>

    <form @submit.prevent="send_message(text)">
        <input name="text" placeholder="Type a message..." autofocus />
        <button type="submit">Send</button>
    </form>
</div>
"""
```

### Form with Validation

```python
# app/live/signup.py
from pydantic import BaseModel, EmailStr, field_validator

class UserForm(BaseModel):
    username: str
    email: EmailStr
    age: int

    @field_validator('username')
    @classmethod
    def username_valid(cls, v: str) -> str:
        if len(v) < 3:
            raise ValueError('Username must be at least 3 characters')
        return v

# Form state
username = ""
email = ""
age = None
errors = {}
submitted = False

def validate_field(field: str, value: str):
    global username, email, age, errors

    # Update field
    if field == "username":
        username = value
    elif field == "email":
        email = value
    elif field == "age":
        age = int(value) if value else None

    # Clear error
    errors.pop(field, None)

    # Validate (simplified)
    try:
        if field == "username" and username:
            UserForm.model_validate({
                "username": username,
                "email": "test@example.com",
                "age": 18
            })
    except Exception as e:
        errors[field] = str(e)

def submit():
    global submitted, errors

    try:
        user = UserForm(
            username=username,
            email=email,
            age=age or 0
        )
        submitted = True
    except Exception as e:
        if hasattr(e, "errors"):
            for error in e.errors():
                field = error["loc"][0]
                errors[field] = error["msg"]

t"""
<div class="signup-form">
    {% if submitted %}
    <div class="success">
        <h2>Welcome, {username}!</h2>
        <p>Check your email to verify your account.</p>
    </div>
    {% else %}
    <form @submit.prevent="submit">
        <h2>Create Account</h2>

        <div class="field {'error' if 'username' in errors else ''}">
            <label>Username</label>
            <input
                name="username"
                value="{username}"
                @input.debounce.300="validate_field('username', value)"
            />
            {% if 'username' in errors %}
            <span class="error-message">{errors['username']}</span>
            {% endif %}
        </div>

        <div class="field {'error' if 'email' in errors else ''}">
            <label>Email</label>
            <input
                type="email"
                name="email"
                value="{email}"
                @input.debounce.300="validate_field('email', value)"
            />
            {% if 'email' in errors %}
            <span class="error-message">{errors['email']}</span>
            {% endif %}
        </div>

        <button type="submit">Create Account</button>
    </form>
    {% endif %}
</div>
"""
```

---

## How State Works

### Per-Connection State (Default)

```python
# app/live/counter.py

count = 0  # Each WebSocket connection gets its own copy
```

**Behind the scenes:**
- When user connects, module is executed in isolated namespace
- Each connection has separate `count` variable
- Changes don't affect other users

### Shared State

```python
# app/live/dashboard.py
from hyper import shared

metrics = shared({"users": 0, "revenue": 0})  # Shared across all connections
```

**Behind the scenes:**
- `shared()` returns a thread-safe proxy
- Changes propagate to all connected clients
- Use `broadcast()` to push updates

### Props (From Parent)

```python
# app/live/user_profile.py

# Props from parent component (read-only)
user_id: int

# Local state (mutable)
bio = ""

def update_bio(new_bio: str):
    global bio
    bio = new_bio
    save_to_db(user_id, bio)

t"""
<div>
    <h1>User {user_id}</h1>
    <textarea @input.debounce="update_bio(value)">{bio}</textarea>
</div>
"""
```

---

## Lifecycle Hooks

Just define these module-level functions:

```python
# app/live/component.py

def on_mount():
    """Called when WebSocket connects"""
    print("User connected!")

def on_unmount():
    """Called when WebSocket disconnects"""
    print("User left!")

def on_error(error: Exception):
    """Called when handler raises exception"""
    print(f"Error: {error}")
```

**Convention:** Function names are special. Framework auto-detects them.

---

## Event Binding

### Basic Events

```python
<button @click="increment">+</button>
<form @submit="save">...</form>
<input @input="search" />
```

### With Arguments

```python
# From form inputs
<form @submit="save(name, email)">
    <input name="name" />
    <input name="email" />
</form>

# From element value
<input @input="search(value)" />

# Literals
<button @click="delete(123)">Delete</button>
<button @click="setColor('red')">Red</button>
```

### With Modifiers

```python
@submit.prevent          # preventDefault()
@click.stop             # stopPropagation()
@input.debounce         # Debounce 300ms
@input.debounce.500     # Debounce 500ms
@click.throttle.100     # Throttle 100ms
```

---

## Comparison: Before & After

### Regular Hyper Component (Non-Live)

```python
# app/components/card.py

title: str
color: str = "blue"

t"""
<div class="card card-{color}">
    <h3>{title}</h3>
    {...}
</div>
"""
```

### Live Component

```python
# app/live/card.py

# Props (from parent)
title: str
color: str = "blue"

# State (mutable)
expanded = False

def toggle():
    global expanded
    expanded = not expanded

t"""
<div class="card card-{color}">
    <h3 @click="toggle">{title}</h3>
    {% if expanded %}
    <div class="content">
        {...}
    </div>
    {% endif %}
</div>
"""
```

**The difference?** Just the directory (`components/` vs `live/`) and event bindings!

---

## Directory Structure

```
app/
├── components/          # Static components (props only)
│   ├── button.py
│   ├── card.py
│   └── header.py
├── live/               # Live components (stateful)
│   ├── counter.py
│   ├── chat.py
│   ├── todos.py
│   └── search.py
└── pages/              # Routes (can use both)
    └── index.py
```

**Using them:**

```python
# app/pages/dashboard.py
from app.components import Header, Card
from app.live import Counter, Chat

t"""
<div>
    <{Header} title="Dashboard" />

    <{Card} title="Stats">
        <{Counter} />
    </{Card}>

    <{Chat} />
</div>
"""
```

---

## Type Safety

Module-level functions are type-checked:

```python
def save(name: str, age: int, admin: bool):
    # Runtime validation:
    # - name must be string
    # - age must be int (auto-converted from form input)
    # - admin must be bool (auto-converted from checkbox)
    pass
```

**Invalid inputs = automatic error:**

```python
# Client sends: {"name": "Alice", "age": "not a number"}
# Server responds: {"error": "Invalid parameter 'age': expected int, got str"}
```

---

## Configuration

### Module-Level Config

```python
# app/live/component.py

# Configure this component
__config__ = {
    "debounce": 300,        # Default debounce for all events
    "transport": "websocket",  # or "sse"
    "fallback": "htmx"      # Fallback when no JS
}

# Rest of component...
count = 0
```

### Global Config

```python
# config.py
LIVE = {
    "transport": "websocket",
    "reconnect_attempts": 5,
    "heartbeat_interval": 30
}
```

---

## Implementation: How It Works

### 1. Component Loading

When framework loads `app/live/counter.py`:

```python
# hyper/live/loader.py

def load_live_component(path: str) -> LiveComponent:
    # Parse module
    module = importlib.import_module(path)

    # Extract state variables (module-level vars)
    state_vars = {
        name: value
        for name, value in vars(module).items()
        if not name.startswith('_') and not callable(value)
    }

    # Extract handlers (module-level functions)
    handlers = {
        name: func
        for name, func in vars(module).items()
        if callable(func) and not name.startswith('_')
    }

    # Extract template (the t-string)
    template = module.__dict__.get('__template__')

    return LiveComponent(
        state=state_vars,
        handlers=handlers,
        template=template
    )
```

### 2. Connection Management

Each WebSocket connection gets isolated namespace:

```python
# hyper/live/connection.py

class LiveConnection:
    def __init__(self, component: LiveComponent, websocket: WebSocket):
        self.component = component
        self.websocket = websocket

        # Create isolated namespace for this connection
        self.namespace = {
            **component.state,  # Copy initial state
            **component.handlers  # Share handlers
        }

    async def handle_event(self, event_name: str, params: dict):
        # Get handler
        handler = self.namespace[event_name]

        # Execute with namespace as globals
        if asyncio.iscoroutinefunction(handler):
            await handler(**params)
        else:
            handler(**params)

        # Re-render with updated namespace
        html = render_template(self.component.template, self.namespace)

        # Send diff to client
        await self.websocket.send_json({
            "type": "update",
            "html": html
        })
```

### 3. Global State

Module-level `global` statement modifies connection namespace:

```python
count = 0  # In namespace: {"count": 0}

def increment():
    global count  # Modifies namespace["count"]
    count += 1
```

This is standard Python! No magic.

---

## Why This Design Wins

### 1. **Feels Like Regular Hyper**

```python
# Regular component
title: str
t"""<h1>{title}</h1>"""

# Live component
title: str
def edit(): pass
t"""<h1 @click="edit">{title}</h1>"""
```

Same structure, just add handlers and events.

### 2. **Zero Boilerplate**

No decorators, no classes, no imports (if using directory convention).

### 3. **Standard Python**

Uses `global` statement for state mutation. Every Python dev knows this.

### 4. **Clear Mental Model**

- Variables = state
- Functions = handlers
- t-string = template

That's it.

### 5. **Type-Safe by Default**

Function signatures = validation schema. No extra work.

---

## Open Question: How to Handle `global`?

The `global` keyword is a bit clunky. Alternatives?

### Option 1: Keep `global` (Standard Python)

```python
count = 0

def increment():
    global count  # Explicit
    count += 1
```

**Pros:** Standard Python, explicit
**Cons:** Verbose, easy to forget

### Option 2: Auto-detect mutations (Magic)

```python
count = 0

def increment():
    count += 1  # Framework detects this is mutation
```

**Pros:** Cleaner syntax
**Cons:** Magic, might confuse linters

### Option 3: Use `state` dict (Explicit)

```python
state = {
    "count": 0
}

def increment():
    state["count"] += 1  # No global needed
```

**Pros:** No `global`, clear namespace
**Cons:** More verbose access

### Option 4: Use `nonlocal` trick (Closures)

```python
# Framework wraps module in function
def _component():
    count = 0

    def increment():
        nonlocal count  # Works like local scope
        count += 1

    t"""..."""
```

**Pros:** `nonlocal` nicer than `global`
**Cons:** Requires wrapping module (tricky)

---

## My Recommendation

**Keep `global` for V1.**

Why?
1. ✅ Standard Python (no magic)
2. ✅ Works with all linters
3. ✅ Explicit (clear intent)
4. ✅ Easy to implement

We can explore alternatives in V2 if users complain.

---

## Summary

**The minimal API:**

1. Put component in `app/live/` directory
2. Use module-level variables for state
3. Use module-level functions for handlers
4. Use top-level t-string for template
5. Use `global` to mutate state
6. Use `@event` attributes for bindings

**That's the whole API.** No decorators, no classes, no ceremony.

```python
# app/live/counter.py

count = 0

def increment():
    global count
    count += 1

t"""
<div>
    <p>{count}</p>
    <button @click="increment">+</button>
</div>
"""
```

**7 lines. Pure Python. Zero boilerplate.**

This is Hyper.
