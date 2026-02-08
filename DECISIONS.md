# Decisions

---

### 001: Yield-Based Streaming

| | |
|--|--|
| **Context** | Need to generate Python from .hyper. Options: return string, return list, or yield. |
| **Decision** | Components are generators that `yield` chunks. `@html` decorator handles str() conversion. |
| **Trade-off** | Generator overhead, but enables streaming responses without API changes. |

---

### 002: @html Decorator

| | |
|--|--|
| **Context** | Without decorator, `str(Template())` returns `"<generator object>"`. |
| **Decision** | `@html` on all components. Built-in to `.hyper` (no import needed). In compiled output: `from hyper import html`. |
| **Trade-off** | Decorator overhead, but uniform API (iterable, str()-able). |

**Usage:**
```python
str(Button(text="Click"))           # full render
for chunk in Button(text="Click"):  # streaming
    response.write(chunk)
```

---

### 003: Keyword-Only Parameters

| | |
|--|--|
| **Context** | Props could be positional or keyword. Positional is error-prone for many props. |
| **Decision** | All params keyword-only via `*,` prefix. `*args` rejected at parse time. |
| **Trade-off** | Verbose call sites, but matches JSX model and allows defaults before non-defaults. |

---

### 004: Escape Markers

| | |
|--|--|
| **Context** | Expressions need HTML escaping. Can't escape at compile time (values unknown). |
| **Decision** | Emit `‹ESCAPE:{expr}›` markers, process via `replace_markers()` at runtime. |
| **Trade-off** | Runtime overhead, but single f-string per block and unified marker system. |

---

### 005: Function Naming — render() + Alias

| | |
|--|--|
| **Context** | Need consistent function naming that works for imports and avoids linter issues. |
| **Decision** | Single-component files: generate `def render()` with alias from filename (`Button = render`). Multi-component files: named functions directly (`def Button`, `def ButtonGroup`). |
| **Trade-off** | Two patterns, but clean imports and linter-compliant. |

---

### 006: File Naming Conventions

| | |
|--|--|
| **Context** | Need to distinguish single-component files from multi-component modules. |
| **Decision** | PascalCase file (`Button.hyper`) = single component, promoted to package level. lowercase file (`buttons.hyper`) = multi-component module, import from submodule. |
| **Trade-off** | Filename matters, but clear intent and Pythonic import patterns. |

**Import patterns:**
```python
from components import Button, Card        # single-component files
from components.buttons import IconButton  # multi-component module
from components.forms import Input, Select # multi-component module
```

---

### 007: Component Invocation Syntax

| | |
|--|--|
| **Context** | Need syntax to use components in templates that's distinct from HTML elements. |
| **Decision** | `<{Component} prop={value} />` for components. Braces indicate "this is Python". |
| **Trade-off** | Slightly more verbose than JSX, but clear distinction between HTML and components. |

**Compiles to:**
```python
yield from Component(prop=value)
```

---

### 008: Slot Syntax

| | |
|--|--|
| **Context** | Components need to accept child content, both default and named slots. |
| **Decision** | `{...}` = default slot, `{...name}` = named slot (in definition). `<{...name}>content</{...name}>` for filling named slots. |
| **Trade-off** | New syntax to learn, but consistent with `{}` = Python convention. |

**Definition:**
```hyper
# Card.hyper
<div class="card">
    <header>{...header}</header>
    <main>{...}</main>
</div>
```

**Usage:**
```hyper
<{Card}>
    <{...header}>
        <h1>Title</h1>
    </{...header}>

    <p>Default slot content</p>
</{Card}>
```

**Slot parameters:** Default slot: `_content`. Named slots: `_header`, `_footer`, etc.

---

### 009: @html Decorator for Local Components

| | |
|--|--|
| **Context** | Need to distinguish between components (yield HTML) and helper functions (return values). |
| **Decision** | `@html` decorator marks functions that produce HTML. Plain `def` = helper function. |
| **Trade-off** | Explicit decorator, but clear intent and no casing rules. |

**Component (yields):**
```hyper
@html
def Badge(text: str):
    <span class="badge">{text}</span>
end

<{Badge} text="New" />
```

**Helper (returns):**
```hyper
def format_price(cents: int) -> str:
    return f"${cents / 100:.2f}"

<span>{format_price(item.price)}</span>
```

**Enforcement:**
- `@html` functions → use with `<{...}>` syntax
- Plain `def` functions → use with `{...}` syntax
- `<{lowercase}>` → Compiler error: "Component names must be PascalCase"

---

### 010: `---` Separator

| | |
|--|--|
| **Context** | Need clear boundary between setup (params, imports, helpers) and template body. |
| **Decision** | `---` required when there's any setup code above. Separates "code" from "output". |
| **Trade-off** | Extra line, but explicit and readable. |

---

### 011: `end` Keyword

| | |
|--|--|
| **Context** | Mixed HTML/Python needs clear block boundaries. Indentation alone is ambiguous. |
| **Decision** | `end` required for control flow blocks (`if`, `for`, `while`, `match`) inside HTML context. Function definitions follow Python rules (end by dedent). |
| **Trade-off** | Hybrid approach - functions are Pythonic, control flow in HTML is explicit. |

**Function definitions (no `end`, use dedent):**
```hyper
@html
def Badge(text: str):
    <span>{text}</span>

@html
def List(items: list):
    <ul>
    for item in items:
        <li>{item}</li>
    end
    </ul>
```

**Control flow needs `end`:**
```hyper
if show:
    <div>Visible</div>
end

for item in items:
    <li>{item}</li>
end
```

**Pure Python helper (standard Python):**
```hyper
def format_price(cents: int) -> str:
    return f"${cents / 100:.2f}"
```

---

### 012: Indentation Rules

| | |
|--|--|
| **Context** | How does indentation interact with `end` keywords? |
| **Decision** | Function definitions use indentation (Python rules). Control flow in HTML uses `end` (indentation optional within). Element tags are self-contained. |
| **Trade-off** | Hybrid approach - familiar for functions, flexible for HTML content. |

**Functions use indentation:**
```hyper
@html
def Badge(text: str):
    <span>{text}</span>
    # dedent ends the function
```

**Control flow in HTML - indentation optional:**
```hyper
# Valid (though discouraged):
if items:
<ul>
for item in items:
<li>{item}</li>
end
</ul>
end
```

**Element tags are self-contained** - no control flow inside `<...>`:
```hyper
<div class={...} />  # ✓ attributes in one unit
```

---

### 013: Conditional Attributes

| | |
|--|--|
| **Context** | Need to conditionally include/omit attributes based on runtime values. |
| **Decision** | Ternary without else: `class={"active" if condition}`. If falsy, attribute is omitted entirely. |
| **Trade-off** | Hyper-specific extension (Python requires else), but cleaner syntax. |

**Examples:**
```hyper
<div class={"active" if is_active}>           # omitted if falsy
<div class={"active" if is_active else ""}>   # empty string if falsy
<div data-id={user_id if user_id}>            # omitted if None/falsy
```

**Compiles to:**
```python
# Attribute conditionally included
**({'class': 'active'} if is_active else {})
```

---

### 014: HTML Validation

| | |
|--|--|
| **Context** | Invalid HTML causes browser quirks and a11y issues. |
| **Decision** | Parser validates: void elements, nesting rules, duplicate attributes. Errors include help text. |
| **Trade-off** | More complex parser, but catches errors early with clear messages. |

---

### 015: Preserve Source Formatting

| | |
|--|--|
| **Context** | Generated Python could collapse content to single lines or preserve structure. |
| **Decision** | Preserve newlines/indentation from source. Use triple quotes. Combine adjacent HTML into single yields until Python/slot breaks it. |
| **Trade-off** | Larger output files, but readable/debuggable code and accurate source maps. |

---

### 016: Template Attribute Expressions

| | |
|--|--|
| **Context** | Attributes like `class="card {theme}"` mix static and dynamic content. |
| **Decision** | `{expr}` inside quoted attributes = template expression, gets ESCAPE markers. `{{` = literal brace. |
| **Trade-off** | Implicit f-string behavior, but intuitive for interpolation. |

**Example:**
```hyper
<div class="card {theme}" data-id="{id}">
```
**Compiles to:**
```python
yield replace_markers(f"""<div class="card ‹ESCAPE:{theme}›" data-id="‹ESCAPE:{id}›">""")
```

---

### 017: Rust Backend Design

| | |
|--|--|
| **Context** | The compiler already parses `.hyper` → AST in Rust. Adding a second codegen backend to emit `.rs` instead of `.py` would give Hyper a server-side Rust target. |
| **Decision** | Rust variant uses Rust types and braces (not indentation/`end`). Props implicitly public, string conversions implicit (`.into()`), variable refs implicit (`{name}` → `&self.name`). Components compile to `struct` + `impl Display`. Slots use buffer strategy initially (render children to `String`). IDE support via ghost macro wrapping (`hyper_component! { ... }` in memory) with span mapping back to source. |
| **Trade-off** | Buffer strategy allocates intermediate strings (not zero-cost), but trivial to implement and still far faster than Python. Generic strategy (`Layout<T: Display>`) deferred. |

See: [`docs/design/rust-backend.md`](docs/design/rust-backend.md)

---

### 018: Module Mode

| | |
|--|--|
| **Context** | Some .hyper files are pure component libraries with no main template. |
| **Decision** | No `---` body = pure module. Use `@html` to define components explicitly. |
| **Trade-off** | Explicit decorators, but clear what's exported. |

**Example:**
```hyper
# buttons.hyper

@html
def Button(text: str):
    <button>{text}</button>
end

@html
def IconButton(icon: str, text: str):
    <button><i class={icon} />{text}</button>
end
```

---
