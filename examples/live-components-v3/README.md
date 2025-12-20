# Live Components V3: _hyperscript Integration

This directory contains examples using **_hyperscript as first-class** with `{}` syntax for server calls.

## Core Philosophy

**Client state (UI, animations) → _hyperscript**
**Server state (data, validation) → Python**

Don't send "dropdown expanded" to server. Keep it on client.
DO send "save user" to server. That needs persistence.

---

## Syntax

### Client-Side Only

```python
t"""
<button _="on click toggle .active">
    Toggle
</button>
"""
```

Pure _hyperscript. No server involved.

---

### Server-Side Call

```python
count = 0

def increment():
    global count
    count += 1

t"""
<button _="on click {increment}">
    +
</button>
"""
```

**`{increment}` = call Python on server**

---

### Mixed (Client + Server)

```python
def save():
    db.save()

t"""
<button _="
    on click
        add .loading to me then
        {save} then
        remove .loading from me then
        add .success to me
">
    Save
</button>
"""
```

**Flow:**
1. Add loading class (client)
2. Call server (async)
3. Remove loading (client)
4. Add success class (client)

---

## Examples

### [counter.py](./counter.py)
Minimal counter with client-side animation + server state
- Server: count value
- Client: pulse animation on click

### [todos.py](./todos.py)
Todo list with mixed state
- Server: todo items (persist)
- Client: fade-out animation, instant strikethrough

### [search.py](./search.py)
Live search with debouncing
- Server: search results
- Client: loading spinner, debouncing

### [chat.py](./chat.py)
Real-time multi-user chat
- Server: messages (shared state)
- Server push: broadcast to all clients
- Client: auto-scroll, fade-in animations

### [dropdown.py](./dropdown.py)
Pure client-side dropdown
- **NO server state**
- **NO WebSocket**
- Just _hyperscript for open/close

### [form_wizard.py](./form_wizard.py)
Multi-step form
- Server: form data + validation
- Client: step navigation, transitions

---

## Key Differences from V2

### V2 (Generic `@event`)
```python
t"""
<button @click="increment">+</button>
"""
```

### V3 (_hyperscript + `{}`)
```python
t"""
<button _="on click {increment}">+</button>
"""
```

**Why V3 is better:**
1. ✅ **One syntax** - just `_` attribute
2. ✅ **_hyperscript ecosystem** - all features available
3. ✅ **Composable** - mix client and server easily
4. ✅ **Clear distinction** - `{}` = server, no `{}` = client
5. ✅ **Progressive enhancement** - _hyperscript degrades gracefully

---

## The `{}` Pattern

**Inside `_` attribute:**
- Regular text = _hyperscript (runs on client)
- `{expression}` = Python call (runs on server, waits for response)

**Examples:**

```html
<!-- Pure client -->
<button _="on click toggle .active">Toggle</button>

<!-- Pure server -->
<button _="on click {save}">Save</button>

<!-- Server with params -->
<button _="on click {delete(123)}">Delete</button>

<!-- From form input -->
<input name="email" />
<button _="on click {subscribe(email)}">Subscribe</button>

<!-- From element value -->
<input _="on input {search(value)}" />

<!-- Mixed -->
<button _="
    on click
        add .saving then
        {save} then
        remove .saving then
        add .saved
">Save</button>
```

---

## Two-Way Binding

### Client → Server
Client triggers server via `{handler}`:

```python
def save(data: str):
    db.save(data)

t"""
<button _="on click {save(value)}">Save</button>
"""
```

### Server → Client
Server pushes updates via `broadcast()`:

```python
from hyper import shared, broadcast

data = shared([])

def refresh():
    data.clear()
    data.extend(fetch_from_db())
    broadcast()  # All clients get update

t"""
<div>
    {% for item in data %}
    <p>{item}</p>
    {% endfor %}

    <button _="on click {refresh}">Refresh</button>
</div>
"""
```

**When `broadcast()` is called:**
1. Server re-renders component
2. Sends HTML diff via WebSocket
3. All connected clients morph DOM
4. _hyperscript `on update from server` events fire

---

## When to Use Server State

**✅ Use server state when:**
- Data must persist (database)
- Data is shared across users
- Validation must be server-side
- Security-sensitive operations
- Business logic

**❌ Don't use server state for:**
- UI toggles (dropdowns, modals)
- Animations
- Local filtering/sorting
- Client-only interactions
- Visual feedback

**Example:**

```python
# app/components/accordion.py (NO server state)

title: str

t"""
<div class="accordion">
    <button _="on click toggle .open on closest .accordion">
        {title}
    </button>
    <div class="content">
        {...}
    </div>
</div>
"""
```

vs.

```python
# app/live/user_profile.py (YES server state)

user_id: int
bio = ""

def save_bio(text: str):
    global bio
    bio = text
    db.update_user(user_id, bio=bio)

t"""
<div>
    <textarea
        _="on input debounced at 1000ms {save_bio(value)}"
    >{bio}</textarea>
</div>
"""
```

---

## _hyperscript Features

All _hyperscript features work with `{}`:

### Debouncing
```html
<input _="on input debounced at 300ms {search(value)}" />
```

### Throttling
```html
<button _="on click throttled at 1000ms {save}">Save</button>
```

### Conditionals
```html
<button _="
    on click
        if #email.value is not empty
            {subscribe(email)}
        end
">Subscribe</button>
```

### Sequences
```html
<button _="
    on click
        add .loading then
        {save} then
        wait 2s then
        remove .loading
">Save</button>
```

### Events
```html
<div _="on update from server
    log 'Component updated!'
    scroll me to the bottom">
    ...
</div>
```

---

## Progressive Enhancement

Components work without JavaScript:

```python
t"""
<form action="/subscribe" method="POST">
    <input name="email" />

    <button _="on click {subscribe(email)}">
        Subscribe
    </button>
</form>
"""
```

**With JS:** WebSocket call, no page reload
**Without JS:** Form POST, traditional request

---

## Implementation Notes

### Template Compilation

```python
# Source
<button _="on click {increment}">+</button>

# Compiled
<button
    _="on click send increment to server"
    data-live-component="counter-abc123"
>+</button>
```

**`{increment}` desugars to `send increment to server`**

### Custom _hyperscript Commands

Added custom commands for server integration:

```javascript
// send <handler> to server
_hyperscript.addCommand("send", ...);

// with loading on <selector>
_hyperscript.addCommand("with", ...);

// debounced at <ms>
_hyperscript.addCommand("debounced", ...);
```

---

## Comparison

| Aspect | V2 (`@event`) | V3 (`_` + `{}`) |
|--------|---------------|-----------------|
| **Syntax** | `@click="handler"` | `_="on click {handler}"` |
| **Client logic** | Limited | Full _hyperscript |
| **Composability** | Hard to mix | Natural mixing |
| **Ecosystem** | New | Existing (_hyperscript) |
| **Learning curve** | New syntax | Reuse _hyperscript knowledge |

---

## The Complete API

1. **Use `_` attribute** for all interactivity
2. **Use `{}` inside `_`** for server calls
3. **Put component in `app/live/`** to enable WebSocket
4. **Use `broadcast()`** for server push
5. **Use `on update from server`** to react to pushes

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

    <button _="
        on click
            add .pulse to me then
            {increment} then
            remove .pulse from me
    ">+</button>
</div>
"""
```

---

**The minimal syntax: just wrap server calls in `{}`**

See [docs/live-state-proposal-v3.md](../../docs/live-state-proposal-v3.md) for full proposal.
