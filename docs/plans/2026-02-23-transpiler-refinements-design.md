# Transpiler Refinements Design

Audit of the current transpiler output, runtime, and API design. Identifies correctness issues, edge cases, and improvements.

---

## A: Replace Markers with Direct Function Calls

### Problem

The marker system (`‹ESCAPE:{expr}›`, `‹CLASS:{expr}›`, etc.) embeds markers in f-strings, then uses `replace_markers()` at runtime to regex-extract values and process them. This has three bugs:

1. **`safe()` is broken.** The f-string stringifies the `Safe` object before `escape_html` sees it, so the `__html__` check never fires. All `safe()` content gets escaped.

2. **`ast.literal_eval` is fragile.** Attribute markers (CLASS, BOOL, STYLE, DATA, ARIA, SPREAD) round-trip Python values through `repr()` → `ast.literal_eval()`. This breaks for any value whose `repr()` isn't a valid Python literal (custom objects, sets, some nested structures).

3. **Regex edge cases.** The `(.+?)` pattern terminates early if a value contains `›`.

### Solution

Replace markers with direct function calls inside f-strings:

```python
# Before
yield replace_markers(f"""<button class=‹CLASS:{class_}› disabled=‹BOOL:{disabled}›>‹ESCAPE:{name}›</button>""")

# After
yield f"""<button class="{render_class(class_)}"{render_attr("disabled", disabled)}>{escape(name)}</button>"""
```

- Content expressions: `‹ESCAPE:{expr}›` → `{escape(expr)}`
- Class attribute: `class=‹CLASS:{expr}›` → `class="{render_class(expr)}"`
- Boolean attribute: `attr=‹BOOL:{expr}›` → `{render_attr("attr", expr)}`
- Style attribute: `style=‹STYLE:{expr}›` → `style="{render_style(expr)}"`
- Data attributes: `data=‹DATA:{expr}›` → `{render_data(expr)}`
- ARIA attributes: `aria=‹ARIA:{expr}›` → `{render_aria(expr)}`
- Spread: `‹SPREAD:{expr}›` → `{spread_attrs(expr)}`

### What gets deleted

- `replace_markers()` function
- `_ATTR_MARKER_PATTERN` and `_ESCAPE_MARKER_PATTERN` regexes
- All `‹` / `›` marker generation in the Rust codegen

### IDE injection impact

The prefix/suffix injection mechanism is unaffected. The injection analyzer computes different offsets (function name length instead of marker length), but the mechanism is identical. The function call approach is actually better for the IDE — `render_class` is a real function that type checkers can resolve.

### `safe()` fix

With direct calls, `escape(safe(html_content))` passes the actual `Safe` object to `escape_html`, which checks `__html__` and returns it unescaped. The round-trip through f-string stringification is eliminated.

---

## B: Rework `@html` Decorator

### Problem

The current `@html` returns a class (`SyncComponentWrapper`), not a function. This breaks:

- `inspect.isfunction()` → False
- `functools.wraps` metadata not preserved
- Type checkers can't see the component's signature
- `isinstance(Button, type)` → True (surprising)

The wrapper also silently accepts positional args: `Button("oops")` sets `_content="oops"` with no error.

### Solution

Return a function wrapper that produces an `HtmlResult` object:

```python
class HtmlResult:
    __slots__ = ("_fn", "_args", "_kwargs")

    def __init__(self, fn, args, kwargs):
        self._fn = fn
        self._args = args
        self._kwargs = kwargs

    def __iter__(self):
        return iter(self._fn(*self._args, **self._kwargs))

    def __str__(self):
        return "".join(self._fn(*self._args, **self._kwargs))

def html(fn):
    @functools.wraps(fn)
    def wrapper(*args, **kwargs):
        return HtmlResult(fn, args, kwargs)
    return wrapper
```

`functools.wraps` preserves `__name__`, `__qualname__`, `__doc__`, `__wrapped__`, and `__signature__`.

Slot handling (`_content`) moves into the wrapper: `_content` is popped from kwargs rather than being a positional arg, so `Button("oops")` raises `TypeError`.

Async variant follows the same pattern with `AsyncHtmlResult` supporting `__aiter__` and `async render()`.

---

## C: Reserved Keyword Handling

### Problem

`class` is a Python reserved keyword. The current compiler renames it to `_class` (leading underscore). Python call sites must use `_class=`, which is non-standard.

### Solution

Use trailing underscore per PEP 8: `class_`. This is the standard Python convention used by SQLAlchemy, Click, and other major libraries.

- `.hyper` source: `class: list` (natural syntax)
- Compiled Python: `class_: list` parameter
- Inter-template calls: `<{Button} class="btn" />` compiles to `Button(class_="btn")` — transparent
- Python call sites: `Button(class_=["btn"])` — recognizable PEP 8 convention

Only `class` needs this treatment. `type` is a soft keyword in Python 3.12+ and works fine as a function parameter name.

---

## D: Slot Mechanism

### Current state

The decorator detects slots by checking if the first parameter is named `_content`. The wrapper class always accepts `_content` as a positional arg.

### Changes

- Slots remain name-based (`_content` for default slot, `_header`/`_footer` for named slots)
- The wrapper rejects unexpected positional args — `_content` is extracted from kwargs, not a positional parameter
- Named slot support: the codegen already generates inner functions for slot content (`def _card(): ...`); named slots follow the same pattern with `_header`, `_sidebar`, etc.

---

## E: Fragments

### Concept

A `fragment` is a named section of a template that renders inline AND is importable standalone. Designed for partial rendering (HTMX, Turbo, etc.).

### Syntax

```hyper
user: User
posts: list[Post]
---
<div class="page">
    fragment Sidebar:
        <aside>
            <h3>{user.name}</h3>
            <p>{user.bio}</p>
        </aside>
    end

    <main>
        for post in posts:
            <article>{post.title}</article>
        end
    </main>
</div>
```

### Compilation

The compiler performs lambda lifting:

1. Finds `fragment Sidebar:` in the body
2. Analyzes captured variables: `user` (doesn't use `posts`)
3. Hoists to module level as `@html def Sidebar(*, user):`
4. Replaces the fragment site with `yield from Sidebar(user=user)`

```python
@html
def Sidebar(*, user):
    yield f"""<aside>
    <h3>{escape(user.name)}</h3>
    <p>{escape(user.bio)}</p>
</aside>"""

@html
def Template(*, user: User, posts: list[Post]):
    yield '<div class="page">'
    yield from Sidebar(user=user)
    yield "<main>"
    for post in posts:
        yield f"""<article>{escape(post.title)}</article>"""
    yield "</main>"
    yield "</div>"
```

### Fragments inside control flow

Valid. The compiler extracts captured variables including loop variables:

```hyper
for post in posts:
    fragment PostCard:
        <article>{post.title}</article>
    end
end
```

Compiles to:

```python
@html
def PostCard(*, post):
    yield f"""<article>{escape(post.title)}</article>"""

@html
def Template(*, posts: list[Post]):
    for post in posts:
        yield from PostCard(post=post)
```

Standalone: `from page import PostCard; str(PostCard(post=some_post))`

### Rules

- Fragment names must be PascalCase (they become importable functions)
- Fragment parameters are keyword-only, untyped (types inferred from usage)
- Fragments cannot be nested inside other fragments
- Duplicate fragment names in the same file are a compile error

---

## F: Remove HTML-in-Expressions

### Problem

The design doc shows `{<span>text</span> if cond}` and `{[<li>{x}</li> for x in items]}`. Implementing this requires a Python expression parser that detects HTML tokens inside arbitrary Python syntax — enormous complexity for marginal DX gain over `for...end` blocks.

### Solution

Remove HTML-in-expressions from the design doc. Simple Python expressions remain supported:

- `{"item" if count == 1 else "items"}` — works (both branches are strings)
- `{count * 2}` — works
- `{user.name.upper()}` — works
- `{[<li>{x}</li> for x in items]}` — removed, use `for...end` instead
- `{<span>text</span> if cond}` — removed, use `if...end` instead
