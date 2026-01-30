# Hyper Design Decisions

## Compiled Output Architecture

### Decision: Yield-based output with @component decorator

**Transpiler outputs yield-style code:**
```python
@component
def MyTemplate(_content=None, *, title="", items=None):
    # <{Card}>
    def _card():
        # <{CardHeader}>
        def _card_header():
            yield f"<h2>{title}</h2>"
        yield from CardHeader(_card_header())
        # </{CardHeader}>
    yield from Card(_card())
    # </{Card}>
```

**Key points:**
- Components are generator functions decorated with `@component`
- Sync components use `def`, `yield`, `yield from`
- Async components (auto-detected if `await` used) use `async def`, `async for ... yield`
- The `@component` decorator enables both yield mode and buffer mode usage
- Comments mirror source structure: `# <{Component}>` and `# </{Component}>`

### Naming conventions:
- Default slot: `_content` (first positional arg) — **only when component has slots**
- Named slots: `_header`, `_sidebar`, etc. (underscore prefix)
- Reserved word props: `_class`, `_for`, `_async` (underscore prefix)
- Regular props: no prefix
- Generator function names: lowercase component name (`_card`, `_card_header`, `_list_item`)

---

## `@component` Decorator Policy

**Decision: `@component` on ALL top-level templates.** No exceptions.

Rationale: without the decorator, `str(Template())` returns `"<generator object>"`.
All `.hyper` templates must have a uniform public API — iterable, context-manageable,
`str()`-able — regardless of complexity. The cost is two lines (import + decorator).
The benefit is zero surprises.

Inner `def` functions do NOT get `@component` — they're local generators consumed
via `yield from` in the same file. They don't need buffer mode or `str()` support.

### `_content` parameter is optional

Components **without slots** don't need `_content`:
```python
@component
def Badge(*, text: str = "", color: str = "blue"):
    yield f'<span class="badge" style="color: {color}">{text}</span>'
```

Components **with slots** have `_content` as first positional arg:
```python
@component
def Card(_content=None, *, title: str = ""):
    yield f'<div class="card"><h1>{title}</h1>'
    if _content is not None:
        yield from _content
    yield '</div>'
```

The `@component` decorator introspects the function signature and handles both cases.

---

## File Structure

```
┌─────────────────────────────────┐
│  HEADER                         │
│  (module-level declarations)    │
├─────────────────────────────────┤
│  ---                            │  ← Required if header has params
├─────────────────────────────────┤
│  BODY                           │
│  (per-render logic + template)  │
└─────────────────────────────────┘
```

---

## `---` Separator

**Decision: `---` is REQUIRED when the header contains parameters.**

This removes all ambiguity about what's a parameter vs what's body code.

| File has... | `---` required? |
|-------------|-----------------|
| Parameters | **Yes** |
| Only imports/constants/defs (module mode) | No |
| Only HTML (no header content) | No |

---

## Header Mode (Above `---`)

The header is a Python module's top-level scope. Anything you'd write at the top
of a `.py` file works here — except mutable assignments and control flow.

### Allowed in Header

| Syntax | What it becomes |
|--------|-----------------|
| `import X` / `from X import Y` | Import (module level) |
| `NAME: Final[type] = expr` | Constant (module level) |
| `type Name = type_expr` (Python 3.12+) | Type alias (module level) |
| `def name(...):` with HTML | `@component` function (module level, exported) |
| `def name(...):` without HTML | Regular function (module level, exported) |
| `async def name(...):` | Same rules as `def` |
| `class Name:` | Class (module level) — no HTML in methods |
| `@dataclass class Name:` | Dataclass (module level) |
| `class Name(Enum):` | Enum (module level) |
| `class Name(Protocol):` | Protocol (module level) |
| `@decorator` | Applied to following def/class |
| `name: type` | Parameter (function signature) |
| `name: type = expr` | Parameter with default |
| `*args: type` | Variadic positional parameter |
| `**kwargs: type` | Variadic keyword parameter |

### NOT Allowed in Header

| Syntax | Why | Error message |
|--------|-----|---------------|
| `name = expr` (untyped) | Ambiguous | "Use `Final[]` for constants or put below `---`" |
| `if`/`for`/`while`/`match`/`try`/`with` | Logic belongs in body | "Control flow belongs in body (below `---`)" |
| HTML (`<tag>`) | Template belongs in body | "HTML belongs in body (below `---`)" |
| HTML inside class methods | Too complex | "HTML not supported in class methods. Use standalone `def`" |

### Header Example

```hyper
from typing import Final, Protocol
from dataclasses import dataclass
from enum import Enum

# Type alias (Python 3.12+)
type ProductDict = dict[str, Any]

# Constants
MAX_ITEMS: Final[int] = 100
COLORS: Final[dict] = {"primary": "blue"}

# Enums
class Status(Enum):
    DRAFT = "draft"
    PUBLISHED = "published"
end

# Dataclasses
@dataclass
class Product:
    name: str
    price: float
end

# Helper functions (no HTML) — exported as regular function
def format_price(amount: float) -> str:
    return f"${amount:.2f}"
end

# Component functions (with HTML) — exported as @component
def Badge(text: str, color: str = "blue"):
    <span class="badge" style="color: {color}">{text}</span>
end

# Parameters (become function signature of main template)
title: str
items: list[Product]
status: Status = Status.DRAFT
---
```

---

## Body Mode (Below `---`)

Everything is allowed. HTML, logic, local definitions, etc.

| Syntax | What it becomes |
|--------|-----------------|
| `<tag>...</tag>` | `yield "..."` |
| `<{Component}>` | `yield from Component(...)` |
| `{expr}` | `‹ESCAPE:{expr}›` in f-string |
| `if`/`for`/`while`/`match`/`try`/`with` | Python control flow + yields |
| `name = expr` | Local variable |
| `name: type = expr` | Typed local variable |
| `def name(...):` with HTML | Local generator (NOT exported) |
| `def name(...):` without HTML | Local helper function |
| `class Name:` | Local class |

### Key Difference: Header vs Body `def`

| Location | `def` with HTML | Can reference params? | Exported? |
|----------|-----------------|----------------------|-----------|
| Header | `@component` at module level | No | Yes |
| Body | Generator (inner function) | Yes (closure) | No |

**Header `def`s are self-contained** — they can't see the template's parameters.
**Body `def`s are closures** — they can reference params and local variables.

---

## Module Mode (No Body)

A `.hyper` file with NO top-level body code (no HTML, no control flow outside defs)
compiles as a pure Python module:

```hyper
# components/badges.hyper — no --- needed, no body

from typing import Final

BADGE_COLORS: Final[dict] = {"info": "blue", "warn": "yellow"}

def Badge(text: str, type: str = "info"):
    <span class="badge" style="color: {BADGE_COLORS[type]}">{text}</span>
end

def Chip(label: str):
    <span class="chip">{label}</span>
end
```

Each `def` with HTML becomes an exported `@component`. Use from other files:

```hyper
from components.badges import Badge, Chip

name: str
---
<div>
    <{Badge} text={name} />
    <{Chip} label={name} />
</div>
```

---

## `end` Keyword

**Decision: `end` is NOT required in the header. Required in the body.**

- Header: Python syntax (indentation-based blocks)
- Body: Hyper syntax (`end` required for blocks)

```hyper
# Header — Python indentation rules
class Status(Enum):
    DRAFT = "draft"
    PUBLISHED = "published"
end  # Still need end for class in header? TBD — might use indentation

title: str
---
# Body — end required
for item in items:
    <li>{item}</li>
end
```

**Note:** Need to decide if `end` is required for classes/defs in header or if we
use Python indentation. For consistency, probably require `end` everywhere in
`.hyper` files, but the header uses Python semantics (no HTML compilation).

---

## Inner Function Compilation

### Rule: HTML always compiles to `yield`, everywhere

If a function body contains HTML nodes, those nodes compile to `yield`.

### Statement calls vs expression calls

| Inner def has HTML? | Called as | Compiled call site | Escape? |
|---------------------|-----------|-------------------|---------|
| Yes | Statement | `yield from fn(args)` | N/A |
| Yes | Expression `{fn()}` | `‹ESCAPE:{"".join(fn(args))}›` | Yes |
| No | Statement | `fn(args)` | N/A |
| No | Expression `{fn()}` | `‹ESCAPE:{fn(args)}›` | Yes |

**`{expr}` ALWAYS escapes.** If you want HTML output from a function, use a
statement call — not an expression.

### Mixed return + yield = compile error

Functions with HTML cannot also have `return value`. The transpiler rejects this.

---

## Slot Fallback Pattern

Generators are always truthy, so `or` doesn't work:

```python
# Wrong — generator is always truthy
if _header or """<h1>Default</h1>""":

# Correct
if _header is not None:
    yield from _header
else:
    yield """<h1>Default</h1>"""
```

---

## Marker System

`replace_markers()` is called per-yield on lines that contain markers:

```python
# Has markers → wrap
yield replace_markers(f"""<div class=‹CLASS:{cls}›>‹ESCAPE:{name}›</div>""")

# Pure static → no wrap
yield """<footer>Copyright 2025</footer>"""
```

---

## TODO

### High Priority

- [ ] **Write golden tests for all inner function scenarios**
- [ ] **Update all existing golden tests to yield format**
- [ ] **Update Rust transpiler to output new format**
  - `@component` decorator on all top-level templates
  - yield-style code
  - `replace_markers()` per-yield
  - Auto-detect async
  - Header defs → module-level `@component` or regular function
  - `Final[]` constants → module level
  - `---` required when params exist
  - Omit `_content` for slot-less components
  - Compile error for HTML in class methods
  - Compile error for mixed return + yield

### Medium Priority

- [ ] **Write tests for @component decorator**
- [ ] **Decide on `end` in header** — require everywhere or use indentation?

### Cleanup

- [ ] `rust/playground/` files → DELETE after proper tests
