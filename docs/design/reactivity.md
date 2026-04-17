# Reactive Templates

> **Status**: Very Early Draft

Reactive templates enable real-time interactivity without JavaScript frameworks. The server manages state. The client applies minimal HTML updates over WebSockets.

---

## Overview

**Current behavior**: Templates return strings.

```python
from components import Counter

html = Counter(count=0)  # Returns full HTML string
```

**Reactive behavior**: Templates maintain state and push updates.

```hyper
# Counter.hyper
count: int = 0

def increment():
    global count
    count += 1

def decrement():
    global count
    count -= 1

---

<div>
    <h1>Count: {count}</h1>
    <button on:click={increment}>+</button>
    <button on:click={decrement}>-</button>
</div>
```

User clicks button. Server increments count. Server sends new HTML. Client morphs the DOM.

**Key principle**: Write Python. Get real-time UI.

---

## Rendering & Updates

### Starting point: full component re-render

The simplest approach that works: when any state changes, re-render the entire component to an HTML string, send it over the WebSocket, and let [idiomorph](https://github.com/bigskysoftware/idiomorph) on the client figure out which DOM nodes actually changed.

```
State change → re-render full component → send HTML → idiomorph patches DOM
```

This is easy to build and easy to debug. The developer writes no update logic. The client morphing library preserves focus, scroll position, and event listeners.

### End goal: automatic fine-grained updates

The developer should never have to think about what re-renders. The engine (ideally in Rust) should automatically know which parts of the template depend on which state, and send only what changed. The developer just writes:

```hyper
count: int = 0
user: User

def increment(self):
    self.count += 1

---

<div>
    <h1>{user.name}'s Dashboard</h1>
    <span>{count}</span>
    <button on:click={increment}>+</button>
</div>
```

When `count` changes, the engine knows only the `<span>` is affected. The `<h1>` is untouched. No manual fragments, no dependency annotations, no reactive primitives. It just works.

**This is an unsolved design problem.** The Rust compiler already has the full AST and knows which expressions reference which variables — so it's well-positioned to build a dependency graph at compile time. But the details of how this works at runtime (granularity of tracked regions, how to handle control flow dependencies, how to represent and transmit diffs) are not yet designed.

### Lifecycle (applies to both approaches)

1. **Initial load**: Full HTML served via standard HTTP (SSR). Tiny `hyper.js` establishes WebSocket.
2. **User interaction**: `hyper.js` sends event over WebSocket (e.g. `{"event": "increment"}`).
3. **Server processes**: Python runs the handler, state changes.
4. **Server determines updates**: Full re-render (v1) or fine-grained engine (future).
5. **Server sends HTML**: Full component string (v1) or minimal patch (future).
6. **Client morphs**: idiomorph patches the DOM. Focus and scroll preserved.

---

## Event Handling

Events trigger server-side Python functions.

### Syntax

```hyper
<button on:click={handler}>Click</button>
```

The compiler generates:

1. A stable ID for the element
2. Client-side JS to send event to server
3. Server-side event dispatcher

### Supported Events

All standard DOM events:

- `click`, `dblclick`
- `submit`, `change`, `input`
- `keydown`, `keyup`, `keypress`
- `mouseenter`, `mouseleave`, `mouseover`
- `focus`, `blur`

Custom events via `on:custom-event`.

### Event Arguments

Access event data in handlers:

```hyper
from hyper import Event

def handle_input(event: Event):
    value = event.target.value
    print(f"User typed: {value}")

---

<input on:input={handle_input} />
```

The server receives serialized event data over WebSocket.

---

## State Management

State lives server-side. Each connected client has isolated state.

### Open question: mutation syntax

Two approaches under consideration:

**Option A: Module-level state with `global`**

```hyper
count: int = 0

def increment():
    global count
    count += 1

---

<span>{count}</span>
<button on:click={increment}>+</button>
```

Pros: Simpler for small components. Matches existing static template style.
Cons: `global` is ugly Python. Compiler may need to rewrite scoping rules.

**Option B: Class-based state with `self`**

```hyper
count: int = 0

def increment(self):
    self.count += 1

---

<span>{count}</span>
<button on:click={increment}>+</button>
```

The compiler treats the header as a class body. In templates, `{count}` still works — the compiler unpacks `self` attributes into the render scope so devs write `{count}` not `{self.count}`.

Pros: Standard Python OOP. No scope magic. Compiler doesn't need to guess what's mutable.
Cons: `self` feels unusual in a template file. More boilerplate for simple cases.

**Decision needed**: Which approach, or both (simple components use global, complex use self)?

### Session Storage

State is stored per WebSocket connection. Sessions live in memory. For persistence, integrate with Redis or database.

---

## Rust/Python Boundary

Hyper already has a Rust compiler for `.hyper` → `.py`. The reactive system extends this with runtime Rust components. The principle: **the developer writes pure Python and HTML. Rust handles the expensive stuff invisibly.**

### Where Rust fits

| Layer | Owner | Why |
|-------|-------|-----|
| `.hyper` compiler | Rust | Fast parsing, file watching, AST analysis, code generation |
| HTML diffing engine | Rust (PyO3) | CPU-bound string comparison is Python's weak spot |
| PubSub fan-out | Rust (optional) | Broadcasting to thousands of sockets is I/O-heavy |
| Network edge | Rust (Phase 2) | Holding idle WebSockets in Rust frees Python for business logic |
| State & business logic | Python | Full ecosystem access, standard ORM/DB libraries |
| Template rendering | Open question (see below) |

### What NOT to do in Rust

Never compile user Python to Rust. Python is too dynamic (`__getattr__`, custom iterators, monkey-patching). Users must keep full access to SQLAlchemy, Pandas, Django ORM, etc. The value proposition is that developers write standard Python.

### Open question: Rust-based template rendering

Python string concatenation for templates is fast enough for most cases (Jinja2 proves this). But there's a possible performance win: **Rust builds the HTML buffer while Python evaluates expressions**.

The pattern ("Rust orchestrates, Python evaluates" via PyO3):

1. The compiler generates a Rust rendering function with the static HTML baked in.
2. For dynamic expressions (`{count}`, `{user.name}`), Rust calls into CPython via PyO3 to evaluate them.
3. Rust writes static parts directly to a pre-allocated buffer and splices in Python values.
4. For control flow (`for item in items`), Rust asks Python for the iterator and loops in Rust, calling back to Python for each item's attributes.

```
Rust: buffer.push("<h1>Hello ")
Rust: → Python: evaluate `name` → "Alice"
Rust: buffer.push("Alice")
Rust: buffer.push("</h1><ul>")
Rust: → Python: iterate `items` →
  Rust: buffer.push("<li>")
  Rust: → Python: evaluate `item.title` → "Post 1"
  Rust: buffer.push("Post 1</li>")
Rust: buffer.push("</ul>")
```

This keeps the developer writing pure Python — Rust just calls into CPython for evaluations using `PyAny::getattr()`, `PyAny::is_truthy()`, etc. Python objects behave exactly as expected (custom `__bool__`, `__iter__`, `__getattr__` all work).

**Trade-off**: FFI crossings add overhead per expression. Worth it for large templates with lots of static HTML. Possibly not worth it for small, expression-heavy templates. Needs benchmarking. Reference: Pydantic v2 uses this exact pattern (Rust core calling into Python) and achieved 5-50x speedups.

---

## Adoption Strategy

### Phase 1: Drop-in library (ASGI adapters)

Hyper ships as a `pip install` package. Developers add it to existing Django/FastAPI/Sanic projects without changing infrastructure.

**Django Channels**:

```python
# routing.py
from hyper.integrations.django import HyperConsumer

websocket_urlpatterns = [
    path("hyper/live/", HyperConsumer.as_asgi()),
]
```

**FastAPI**:

```python
from hyper.integrations.fastapi import HyperRouter

app.include_router(HyperRouter, prefix="/hyper")
```

The host framework handles WebSocket connections. Hyper just processes events and returns HTML patches. No separate server process.

**PubSub in Phase 1**: Abstraction layer with pluggable backends.
- `MemoryBroker` — default for development (single process).
- `RedisBroker` / `NATSBroker` — for multi-worker production deployments.

### Phase 2: Standalone framework

Once adoption grows, offer `hyper-server` powered by Granian (Rust ASGI server):

```bash
pip install hyper-server
hyper serve app.py
```

Benefits over standard ASGI servers:
- **Rust holds idle WebSockets**: Thousands of connections with minimal RAM. Python only wakes when a user actually interacts.
- **RSGI protocol**: Granian's alternative to ASGI that bypasses Python dictionary creation for request/response, enabling zero-copy data transfer.
- **Built-in PubSub**: In-memory Rust broadcast channels replace Redis for single-server deployments.

Users can migrate from Phase 1 → Phase 2 by changing their run command. No code changes.

---

## Scaling Considerations

### Python 3.14 Free-Threaded Mode

Python 3.14 made the no-GIL build officially supported. This changes the deployment model:

**Old (with GIL)**: Run multiple processes (`uvicorn --workers 4`) to use all CPU cores. Each process has isolated memory — in-memory PubSub and shared state can't work across workers without Redis.

**New (no-GIL)**: Single process, multiple threads, all sharing the same memory. This enables:
- In-memory PubSub without Redis (all threads see the same broadcast channels)
- Shared embedded database (SQLite/LibSQL in-process, no network round-trip)
- Simpler deployment (`python app.py` — one process, all cores)

**Trade-off**: Developers must be careful with shared mutable state (race conditions). Hyper should provide safe abstractions — each user's component state is isolated, shared state goes through the embedded DB which handles locking.

### Session Distribution

For multi-server deployments, use Redis for distributed sessions and PubSub.

### Connection Limits

- Use async/await for non-blocking I/O
- Implement heartbeat/ping for connection health
- Set idle timeouts for inactive sessions
- Phase 2: Granian holds idle connections in Rust, dramatically reducing Python memory pressure

---

## Comparison with LiveView

| Feature | Phoenix LiveView | Hyper Reactive |
|---------|-----------------|----------------|
| Language | Elixir | Python |
| State location | Server | Server |
| Transport | WebSocket | WebSocket |
| Rendering | Re-render + diff | Re-render fragments + morph |
| Client code | Standalone JS library | Auto-generated / tiny library |
| Framework lock-in | Phoenix only | Any Python framework (Phase 1) |

**Elixir advantage**: BEAM VM gives lightweight processes, built-in clustering, and fault isolation per connection. Hyper must engineer these (Python threads + Redis for distribution).

**Hyper advantage**: Python ecosystem (ORMs, ML libraries, etc.) and drop-in integration with existing projects.

---

## Open Questions

### Event syntax

```hyper
<button on:click={handler}>Click</button>     # Svelte-style (current)
<button @click={handler}>Click</button>        # Vue-style
<button onclick={handler}>Click</button>       # HTML-style
```

**Decision needed**: Which syntax feels most natural for Python developers?

### State persistence

Should state automatically persist to database/Redis?

**Decision needed**: Explicit or implicit persistence?

### Multi-client sync

Should multiple browser tabs for same user share state?

**Decision needed**: Isolated sessions or synchronized state?

### Progressive enhancement

Should reactive templates work without JavaScript (fallback to form submission)?

**Decision needed**: Require JavaScript or provide fallback?

---

**See Also**:
- [Templates](templates.md) - Template syntax
- [Template Implementation](../implementation/templates.md) - Transpiler details
- [SSR](ssr.md) - Server-side rendering
