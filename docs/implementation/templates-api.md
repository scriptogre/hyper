# Templates API

Internal API for template compilation and rendering.

---

## Principles

1. **Structure at compile time, values at runtime.** HTML parsing happens once during compilation. Runtime just interpolates values and concatenates strings.

2. **Generated code is readable Python.** The compiler produces code you can inspect and debug.

3. **Minimal runtime dependencies.** Just escape functions and attribute formatters, no tree building.

---

## Architecture

```
Template Source (.py)
        ↓
    Parser (reuse tdom parser)
        ↓
    Tree (TNode from tdom/nodes.py)
        ↓
    CodeGen (walk tree, emit Python)
        ↓
    Compiled Code (Python functions)
        ↓
    .pyi Stubs (for IDE)
```

**What happens at each phase:**

| Phase | What | When |
|-------|------|------|
| Parse | Extract props, parse HTML to TNode tree | Compile time |
| CodeGen | Walk tree, generate Python functions | Compile time |
| Load | Import hook compiles if needed, loads module | Import time |
| Execute | Call generated function, interpolate values | Request time |

---

## Compiler

```python
from hyper.templates import compile

module = compile(
    source: str,
    path: Path,
) -> CompiledTemplate

stub = generate_stub(module) -> str
```

---

## CompiledTemplate

```python
@dataclass
class CompiledTemplate:
    source: str                           # Generated Python source
    render_func: Callable                 # Main render function
    fragments: dict[str, Callable]        # Fragment functions
    stream_func: Callable | None          # Streaming variant if applicable
    metadata: TemplateMetadata
```

---

## TemplateMetadata

```python
@dataclass
class TemplateMetadata:
    props: dict[str, Prop]
    fragments: dict[str, Fragment]
    is_async: bool
    has_streaming: bool
```

---

## Prop

```python
MISSING = object()

@dataclass
class Prop:
    name: str
    type_hint: type | None
    default: Any = MISSING  # MISSING means required
```

---

## Fragment

```python
@dataclass
class Fragment:
    name: str
    deps: frozenset[str]    # Props this fragment uses
    is_async: bool
```

---

## Runtime Utilities

Simple functions called by generated code. No tree building, just value processing.

```python
# hyper/templates/runtime.py
from typing import Any
from markupsafe import Markup

def escape_html(value: Any) -> str:
    """Escape value for safe HTML output.

    - Markup objects: pass through unchanged
    - None: empty string
    - Other: str() then escape
    """

def format_classes(*values: str | list | dict | None) -> str:
    """Convert class values to space-separated string.

    Accepts:
        - str: "btn primary"
        - list: ["btn", "primary"]
        - dict: {"active": True, "disabled": False} → "active"
        - Nested combinations

    Example:
        format_classes("btn", ["large"], {"disabled": False})
        # Returns: "btn large"
    """

def format_styles(styles: dict[str, str | None]) -> str:
    """Convert style dict to CSS string.

    Example:
        format_styles({"color": "red", "margin": None})
        # Returns: "color: red"
    """

def format_attrs(attrs: dict[str, Any]) -> str:
    """Convert attribute dict to HTML attribute string.

    - True: attribute without value (e.g., disabled)
    - False/None: omit attribute
    - Other: name="value"

    Example:
        format_attrs({"disabled": True, "id": "main", "hidden": False})
        # Returns: 'disabled id="main"'
    """

def join_children(children: tuple) -> str:
    """Join children tuple to string."""
    return ''.join(str(c) for c in children)
```

**Note:** data-* attribute special handling (JSON serialization, flattening) is deferred for later. Currently treated as regular attributes.

---

## IDE Support via Stubs

Generated `.pyi` stub files provide type information to IDEs.

### Method 1: Inline (Recommended)

```
app/
  pages/
    Dashboard.py      # Template source (committed)
    Dashboard.pyi     # Generated stub (gitignored)
```

**.gitignore:**
```
**/*.pyi
```

**How it works:**
- Stub lives next to source
- IDE finds it automatically, no config needed
- Gitignored, so not in repo

**Pros:**
- Zero configuration
- Works in all editors
- Clean git history

**Cons:**
- Visible in file tree (but faded/grayed as ignored)

---

### Method 2: Centralized

```
.hyper/
  stubs/
    app/
      pages/
        Dashboard.pyi
app/
  pages/
    Dashboard.py
```

**pyproject.toml:**
```toml
[tool.pyright]
stubPath = ".hyper/stubs"

[tool.mypy]
mypy_path = ".hyper/stubs"
```

**Setup:**
```bash
hyper init  # Adds config to pyproject.toml
```

**How it works:**
- Stubs in hidden folder
- Editor reads config from pyproject.toml
- Cleaner project structure

**Pros:**
- Hidden from file tree
- Organized in one place

**Cons:**
- Requires pyproject.toml config
- Some editors may need manual "mark as sources root"

---

## Stub Format

```python
# Dashboard.pyi
from __future__ import annotations
from typing import AsyncIterator
from app.models import User, Post

async def __call__(user: User, posts: list[Post]) -> str: ...
async def render(user: User, posts: list[Post]) -> str: ...

def sidebar(user: User) -> str: ...
async def content(posts: list[Post]) -> str: ...

def stream(user: User, posts: list[Post]) -> AsyncIterator[str]: ...

__generated__: str  # The compiled source code
```

---

## Import Hook

```python
# In package __init__.py
from hyper.templates import enable_templates

enable_templates()
```

**What it does:**
1. Intercepts imports for `.py` files in this package
2. Detects if file is a template (has `t"""` string)
3. Compiles if needed (checks cache)
4. Generates `.pyi` stub
5. Loads compiled code into `sys.modules`

---

## Debug Configuration

```python
from hyper.templates import configure

configure(debug=True)   # Hot reload, recompile on change
configure(debug=False)  # Production, compile once
```

Or via environment:
```bash
DEBUG=true python app.py
```

Default: reads `DEBUG` env var, overridable with `configure()`.

---

## Inspecting Compiled Code

```python
# As property on template
from app.pages import Dashboard
print(Dashboard.__generated__)

# As function
from hyper.templates import inspect
print(inspect("app/pages/Dashboard.py"))
```

**CLI:**
```bash
hyper inspect app/pages/Dashboard.py
```

---

## Compilation Example

**Source (Dashboard.py):**
```python
user: User
posts: list[Post]

t"""
<div id="sidebar" {'fragment'}>{user.name}</div>
<main id="content" {'fragment'}>
  <!--@ for p in {posts} -->
    <article>{p.title}</article>
  <!--@ end -->
</main>
"""
```

**Generated Python:**
```python
from hyper.templates.runtime import escape_html

async def render(user: User, posts: list[Post]) -> str:
    return sidebar(user) + await content(posts)

def sidebar(user: User) -> str:
    return f'<div id="sidebar">{escape_html(user.name)}</div>'

async def content(posts: list[Post]) -> str:
    items = ''.join(
        f'<article>{escape_html(p.title)}</article>'
        for p in posts
    )
    return f'<main id="content">{items}</main>'

# Public API
__call__ = render
__generated__ = """..."""  # This source code
```

**Generated Stub (Dashboard.pyi):**
```python
from app.models import User, Post

async def __call__(user: User, posts: list[Post]) -> str: ...
async def render(user: User, posts: list[Post]) -> str: ...
def sidebar(user: User) -> str: ...
async def content(posts: list[Post]) -> str: ...

__generated__: str
```

---

## Cache

```python
from hyper.templates import cache

cache.get(path: Path) -> CompiledTemplate | None
cache.set(path: Path, compiled: CompiledTemplate) -> None
cache.invalidate(path: Path) -> None
cache.clear() -> None
```

---

## Error Handling

```python
class TemplateError(Exception):
    path: Path
    line: int
    column: int
    message: str

class TemplateSyntaxError(TemplateError): ...
class FragmentError(TemplateError): ...
```

Errors follow `docs/standards/error-messages.md`.

Example:
```
Fragment attribute requires an id.

  You wrote:
    <div {'fragment'}>...</div>

  Add an id:
    <div id="sidebar" {'fragment'}>...</div>
```

---

## What Gets Reused from tdom

| Component | Use |
|-----------|-----|
| `parser.py` | Parse HTML at compile time |
| `nodes.py` | TNode tree structure at compile time |
| `escaping.py` | Move functions to runtime utilities |
| `classnames.py` | Move to runtime utilities |
| `processor.py` | **Not needed** - replaced by generated code |

---

## Summary

| Aspect | Decision |
|--------|----------|
| Compilation model | Compile-time structure, runtime values |
| Runtime dependencies | Minimal: escape + format utilities |
| IDE support | .pyi stubs (inline or centralized) |
| Debug | `.__generated__` property + `hyper inspect` |
| Configuration | `configure()` function + DEBUG env var |
| Import | `enable_templates()` in package `__init__.py` |
