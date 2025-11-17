# HyperLive - Real-Time Server-Driven UI

**Goal:** Build the foundation for next-gen hypermedia-driven applications (HDAs). Learn from Livewire/LiveView, avoid their mistakes, exceed their capabilities.

---

## What Livewire Does

**Core concept:** Server-side state + automatic DOM updates without JavaScript

**Flow:**
1. User interaction → AJAX request with state + checksum
2. Server hydrates component, runs action, re-renders
3. Server diffs HTML, returns only changes
4. Client morphs DOM surgically

**Key innovations:**
- **Two-way data binding** via `wire:model` (syncs input ↔ server property)
- **Automatic state persistence** across requests (serverMemo)
- **Security via checksums** (HMAC prevents tampering)
- **DOM morphing** (preserves focus, inputs, client state)
- **Dirty tracking** (only sends changed properties)
- **HTML hashing** (skips unchanged HTML)

**Technical details:**
- Metadata embedded in HTML: `wire:id`, `wire:initial-data`
- Hydration middleware pipeline: validate checksum → hydrate props → call method → render → diff
- Client scans for `[wire:id]`, registers event listeners
- Uses `morphdom` library for intelligent DOM patching

---

## What LiveView Does Better

**Uses WebSockets instead of AJAX:**
- Persistent connection (low latency)
- Stateful server process per client
- Bidirectional streaming

**BEAM/Erlang advantages:**
- Handles 10,000s of concurrent connections
- Lightweight processes (not threads)
- Fault tolerance built-in
- OTP supervision trees

**Result:** More efficient for high-frequency updates

---

## Livewire's Downsides

1. **Chatty AJAX** - Every interaction = full request/response cycle
2. **Server load** - Re-renders entire component every time
3. **Latency** - Network round-trip for every update
4. **No streaming** - Can't push updates proactively
5. **PHP/Laravel-only** - Not framework-agnostic
6. **Proprietary format** - Can't use with other clients

---

## LiveView's Downsides

1. **Elixir/Phoenix-only** - Locked into ecosystem
2. **WebSocket overhead** - Overkill for simple cases
3. **Complexity** - Requires understanding BEAM, processes, OTP
4. **No HTTP fallback** - Must use WebSockets

---

## HyperLive Goals

### 1. Transport Agnostic
- Work with AJAX (simple case)
- Work with WebSockets (real-time case)
- Work with SSE (server→client push)
- Work with WebTransport (future)

### 2. Framework Agnostic
- Server outputs universal patch format
- Client adapters for HTMX, Alpine, vanilla JS, etc.
- Not Python-specific (design can port to any language)

### 3. Progressive Enhancement
- Start with AJAX (works everywhere)
- Upgrade to WebSocket if available
- Degrade gracefully

### 4. Minimal Overhead
- Don't re-render entire page
- Compute actual diff on server
- Send only `<hx-partial>` tags for changes
- Use HTMX 4.0 built-in morphing

### 5. No Component Boilerplate
- Just regular Hyper pages
- Module-level variables = state
- No class inheritance needed

---

## Core Mechanisms

### State Serialization

**Store in session/Redis:**
```python
{
  "page_id": "counter-abc",
  "checksum": "hmac_hash",
  "state": {"count": 0},
  "html_hash": "402ed05a"
}
```

**Serialize smartly:**
- Primitives: direct
- Models: store ID + type metadata (hydrate on request)
- Collections: store IDs array

### Security (Livewire model)

**Generate checksum:**
```python
checksum = hmac(secret, page_id + state + fingerprint)
```

**Validate every request:**
- Reject if checksum mismatch
- Prevents client tampering

### Hydration Pipeline

**Incoming request:**
1. Validate checksum
2. Restore state from session
3. Set module variables
4. Run page logic (GET/POST block)
5. Render template
6. Diff old vs new HTML
7. Generate `<hx-partial>` tags
8. Return minimal response

**Middleware stack:**
```python
ValidateChecksum
→ HydrateState
→ RunPageLogic
→ RenderTemplate
→ ComputeDiff
→ GeneratePartials
→ UpdateSession
```

### Diffing Algorithm

**ID-based (fast):**
- Mark elements with IDs
- Compare only elements with IDs
- Generate partial for each changed ID

**Tree-based (thorough):**
- Walk entire DOM tree
- Find minimal set of changes
- More accurate, slower

**Hybrid:**
- Use IDs when available
- Fall back to tree diff for unmarked sections

### Output Format

**HTMX 4.0 `<hx-partial>` tags:**
```html
<hx-partial hx-target="#count" hx-swap="morph">
  <h1 id="count">Count: 1</h1>
</hx-partial>
```

**Why this format:**
- HTMX 4.0 native support
- Other clients can parse `hx-target`, `hx-swap`, content
- Human-readable
- Debuggable

---

## Transport Strategies

### Phase 1: AJAX (Start Here)
- Stateless, session-based state
- Works everywhere (100% compatibility)
- Simple to implement and debug
- Good for forms, occasional updates
- 100-300ms latency acceptable

**Implementation priority: NOW**

### Phase 2: WebSocket (Real-Time Upgrade)
- Stateful, in-memory state per connection
- Persistent bidirectional connection
- Server can push updates proactively
- 20-50ms latency
- Good for chat, dashboards, collaborative editing

**Implementation priority: AFTER AJAX WORKS**

### Phase 3: WebTransport (Future Optimization)
- HTTP/3 + QUIC protocol
- Native multiplexing (multiple streams per connection)
- Connection migration (survives network changes)
- 10-30ms latency
- Good for gaming, high-frequency trading

**Implementation priority: LATER (when performance matters)**

**Progressive enhancement (automatic):**
```python
# Framework detects best transport
if websocket_available():
    use_websocket()  # Phase 2
else:
    use_ajax()       # Phase 1 (fallback)

# Phase 3 added later:
# if webtransport_available():
#     use_webtransport()
```

---

## Example: Counter

```python
# app/pages/counter/index.py
from hyper import GET, POST

count: int = 0  # State variable

if GET:
    t'''
    <div hx-wt:connect="/counter">
        <h1 id="count">Count: {count}</h1>
        <button hx-wt:send>+</button>
    </div>
    '''

elif POST:
    count += 1
    # Hyper auto-diffs, returns:
    # <hx-partial hx-target="#count" hx-swap="morph">
    #   <h1 id="count">Count: 1</h1>
    # </hx-partial>
```

**What happens:**
1. Initial GET: Full page + state saved to session
2. Click button: POST request includes page_id + checksum
3. Hyper restores `count` from session
4. Runs `count += 1`
5. Re-renders template
6. Diffs: only `#count` changed
7. Returns `<hx-partial>` tag
8. HTMX morphs `#count` element

---

## Example: Live Search

```python
# app/pages/search/index.py
from hyper import GET

q: str = ""
results: list = []

if GET:
    if q:
        results = search_db(q)

    t'''
    <div hx-ext="hyperlive">
        <input name="q"
               value="{q}"
               hx-get="/search"
               hx-trigger="keyup changed delay:300ms"
               hx-include="this">

        <div id="results">
            {[t'<div>{r.title}</div>' for r in results]}
        </div>
    </div>
    '''
```

**Each keystroke:**
- Sends `q` value
- Hyper updates state
- Re-renders
- Diffs: only `#results` changed
- Returns `<hx-partial hx-target="#results">`

---

## Example: Form Validation

```python
# app/pages/signup/index.py
from hyper import GET, POST, Form
from typing import Annotated

errors: dict = {}

if GET:
    t'''
    <form hx-post="/signup" hx-ext="hyperlive">
        <input name="email" />
        <span id="email-error">{errors.get("email", "")}</span>

        <input name="password" />
        <span id="password-error">{errors.get("password", "")}</span>

        <button>Sign Up</button>
    </form>
    '''

elif POST:
    email: Annotated[str, Form()]
    password: Annotated[str, Form()]

    if not email:
        errors["email"] = "Required"
    if len(password) < 8:
        errors["password"] = "Too short"

    # Returns only changed error spans:
    # <hx-partial hx-target="#email-error">...</hx-partial>
```

---

## WebSocket Mode

```python
# Same page code, but with WebSocket:
# Client sends: ws://localhost:8000/counter
# Server maintains connection
# Can push updates without request:

async def push_update():
    count += 1
    partial = compute_diff()
    await websocket.send(partial)
```

**Benefits:**
- Real-time updates
- Server can push
- Lower latency
- Persistent state in memory

**Use cases:**
- Live dashboards
- Chat applications
- Collaborative editing
- Real-time notifications

---

## Key Innovations Over Livewire/LiveView

1. **Transport flexibility** - Not locked to AJAX or WebSocket
2. **Framework agnostic** - Universal patch format
3. **Minimal payload** - Send `<hx-partial>` tags, not full HTML + dirty array
4. **No component classes** - Works with regular pages
5. **HTMX 4.0 native** - Use built-in morphing
6. **Progressive enhancement** - AJAX → WebSocket upgrade
7. **Python-first** - But design is portable

---

## Open Questions

1. **State storage** - Session vs Redis vs in-memory?
2. **Concurrency** - How handle simultaneous updates?
3. **State cleanup** - When expire old state?
4. **Large state** - Size limits? Compression?
5. **Nested updates** - How handle page components?
6. **Opt-in vs opt-out** - Auto-enable or explicit decorator?
7. **Diffing algorithm** - Speed vs accuracy tradeoff?
8. **WebSocket scaling** - How many connections per server?

---

## Technical Decisions Needed

**State serialization:**
- [ ] Choose storage backend (session/Redis/hybrid)
- [ ] Design metadata format for complex types
- [ ] Handle circular references

**Diffing:**
- [ ] Implement ID-based differ
- [ ] Implement tree-based differ
- [ ] Benchmark performance
- [ ] Choose default strategy

**Transport:**
- [ ] AJAX implementation
- [ ] WebSocket upgrade mechanism
- [ ] SSE for server push
- [ ] WebTransport (future)

**Security:**
- [ ] Checksum algorithm (HMAC-SHA256?)
- [ ] Fingerprint format
- [ ] Prevent replay attacks
- [ ] Rate limiting

**Client:**
- [ ] HTMX extension for metadata
- [ ] WebSocket reconnection logic
- [ ] Fallback mechanisms
- [ ] Error handling

---

## Next Steps

1. **Prototype state serialization** - Prove we can hydrate/dehydrate
2. **Build simple differ** - ID-based, outputs `<hx-partial>` tags
3. **Implement checksum validation** - Security first
4. **Create HTMX extension** - Auto-send page_id + checksum
5. **Test with counter example** - Validate full flow
6. **Benchmark diff performance** - Measure overhead
7. **Add WebSocket support** - Prove upgrade path works
8. **Document patterns** - Best practices emerge

---

## Success Metrics

- **Bandwidth** - 10x less than Livewire (send partials, not full HTML)
- **Latency** - <100ms for simple updates (AJAX), <50ms (WebSocket)
- **Scalability** - Handle 1000s of concurrent connections
- **Developer experience** - Zero boilerplate, just works
- **Framework adoption** - Other Python frameworks can use it

---

## References

- [Livewire deep dive](https://calebporzio.com/how-livewire-works-a-deep-dive)
- [LiveView guide](https://hexdocs.pm/phoenix_live_view)
- [morphdom](https://github.com/patrick-steele-idem/morphdom)
- [idiomorph](https://github.com/bigskysoftware/idiomorph)
- HTMX 4.0 `<hx-partial>` spec (forthcoming)
- WebTransport spec

---

## Future Enhancements

### WebTransport (Phase 3)

**When AJAX + WebSocket aren't enough:**

WebTransport (HTTP/3 over QUIC) provides additional benefits for specific use cases:

**Key advantages:**
1. **Native multiplexing** - Multiple streams per connection (no head-of-line blocking)
2. **Connection migration** - Survives WiFi → cellular switches without reconnect
3. **0-RTT resumption** - Instant reconnection for returning clients
4. **10-30ms latency** - vs 20-50ms for WebSocket, 100-300ms for AJAX

**Use cases:**
- Real-time collaborative editing (Google Docs-style)
- Gaming or trading applications
- Mobile-first apps where connection migration critical
- High-frequency dashboards (multiple independent widgets)

**Implementation approach:**
- Same developer API (no code changes)
- Framework automatically uses WebTransport when available
- Falls back to WebSocket → AJAX gracefully
- Requires parallel server pattern (aioquic) OR Hypercorn fork

**Browser support:** 95%+ (Chrome 97+, Safari 18+, Firefox 123+)

**Decision:** Implement AFTER AJAX and WebSocket work well. Most apps won't need this.

---

**Status:** Concept phase. Everything subject to change.
