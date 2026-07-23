# Template Compiler

This document defines the approved compiler target. Open decisions remain in [the implementation plan](component-language-plan.md).

Components compile to Python generator functions. Component libraries compile to Python modules.

## Basic Example

Write this:

```hyper
name: str
count: int = 0

---

<div>
    <h1>Hello {name}</h1>
    if count > 0:
        <p>You have {count} items</p>
    end
</div>
```

Get this:

```python
from hyperhtml import component, escape


@component
def Greeting(
        *,
        name: str,
        count: int = 0,
):
    yield """<div>"""
    yield f"""<h1>Hello {escape(name)}</h1>"""
    if count > 0:
        yield f"""<p>You have {escape(count)} items</p>"""
    yield """</div>"""
```

The `@component` decorator returns a callable `Component`. Calls buffer output; `.stream()` exposes generated chunks.

Compile once. Render many times.

---

## The `---` Delimiter

A `.hyper` file is either an implicit component or a component library. The `---` separates an implicit component's header from its rendering code. A declaration-only library has no implicit rendering function.

### Above `---`: Header

Type hints become keyword-only function parameters:

```hyper
title: str
count: int = 0
---
<div>{title}</div>
```

```python
from hyperhtml import component, escape


@component
def Page(
        *,
        title: str,
        count: int = 0,
):
    yield f"""<div>{escape(title)}</div>"""
```

Spread names like `kwargs`, `props`, `rest`, `attrs`, and `attributes` are auto-injected into the signature:

```hyper
title: str
---
<div {**props}>{title}</div>
```

```python
@component
def Page(
        *,
        title: str,
        **props,
):
    yield f"""<div{spread_attrs(props)}>{escape(title)}</div>"""
```

### Below `---`: Function Body

Python statements compile into the component function. HTML emits output. `{expr}` emits an escaped value.

```hyper
---
items = ["Apple", "Banana", "Cherry"]

<ul>
    for item in items:
        <li>{item}</li>
    end
</ul>
```

```python
@component
def ItemsLoop():
    items = ["Apple", "Banana", "Cherry"]
    yield """<ul>"""
    for item in items:
        yield f"""<li>{escape(item)}</li>"""
    yield """</ul>"""
```

### Blocks and `end`

A compound statement whose content follows on indented lines requires an aligned `end` in every file zone:

```hyper
for item in items:
    <li>{item}</li>
end

if is_active:
    <span>Active</span>
end
```

Indentation defines the contents. `end` closes the statement. Dedentation alone never closes it.

A compound statement owns one `end`. Its `elif`, `else`, `except`, `finally`, and `case` clauses do not have separate endings.

Short content may follow the outer colon on the same logical line and does not use `end`:

```hyper
if is_active: <span>Active</span>
component Divider(): <hr />
```

Same-line content selects Python or template context once. Python semicolons remain Python separators; semicolons in template output remain text. Python and template output cannot mix on the same line.

The tokenizer finds the outer colon while ignoring colons inside strings, annotations, dictionaries, patterns, slices, lambdas, and brackets.

Structural indentation uses spaces. Tabs are rejected. `end` may have a trailing Python comment.

The tree builder uses one indentation-aware block parser for headers, rendering code, branch clauses, and component declarations. Structural indentation metadata is separate from rendered whitespace.

### Add Imports

Imports go above `---` and remain module-level:

```hyper
from app.components import Button
title: str
---
<{Button} label={title} />
```

### When `---` Is Optional

**Only HTML:**

```hyper
<div>Hello</div>
```

No `---` needed. Everything is template body.

**Only definitions:**

```hyper
def helper():
    return "value"
end

class Value:
    pass
end
```

No `---` needed. Everything is module-level. No render function generated. This is a library file.

**Mixed content needs `---`:**

```hyper
def helper():
    return "value"
end
---
<div>{helper()}</div>
```

---

## Explicit Components

`component` parses into a component-definition AST node. A normal `def` remains a Python definition and cannot contain HTML.

```hyper
component Badge(*, text: str):
    <span>{text}</span>
end

component Divider(): <hr />
```

The generator emits an `@component`-decorated Python function. The decorator returns a `Component` while preserving the generated function's name and signature.

**Coming soon:** `@render_here` will export a subcomponent and render it at its declaration position:

```hyper
title: str
---
@render_here
component Header(*, title: str):
    <header>{title}</header>
end
```

Subcomponents are defined first and attached by name:

```python
@component
def Header(*, title: str):
    yield f"""<header>{escape(title)}</header>"""


@component(subcomponents=[Header])
def Page(*, title: str):
    yield from Header.stream(title=title)
```

`Page.Header` exposes the same read-only `Header` component.

### Slot Parameters

Default and named slots become keyword-only component arguments:

```hyper
component Layout(*, title: str):
    <header>{...header}</header>
    <main>{...}</main>
end
```

```python
from collections.abc import Iterable


@component
def Layout(
        *,
        title: str,
        content: Iterable[str] | None = None,
        header: Iterable[str] | None = None,
):
    yield """<header>"""
    if header is not None:
        yield from header
    yield """</header><main>"""
    if content is not None:
        yield from content
    yield """</main>"""
```

`content` is reserved for the default slot. Named slots use their source names. A prop cannot use `content` or share a named slot's name. These collisions are compile errors.

A bare `return` remains a bare generated return. Scope-aware validation rejects `return value` and explicit `yield` only in the active component. Nested normal functions retain Python return and yield behavior.

`async component` is explicit. Implicit components infer async from `await`, `async for`, or `async with` in their own rendering scope.

`<>...</>` parses as a transparent fragment node and emits only its children.

## Compiler Pipeline

```text
Source → Parse → Lower → Plugins → Generate → Map
```

### Parser

Builds the source tree and validates its structure.

### Lowering and Plugins

Lowering creates the compiler AST. Plugins transform it, collect metadata, and validate component scopes.

### Generator: Yield-Based Streaming

The generator emits `yield` statements instead of appending to a list. Static HTML yields plain strings. Dynamic content yields f-strings with direct function calls for escaping and attribute rendering.

Special attributes use helper functions:

```hyper
classes = ["btn", "active"]
<div class={classes}>content</div>
```

```python
classes = ["btn", "active"]
yield f"""<div class="{render_class(classes)}">content</div>"""
```

Content expressions use `escape()`, class attributes use `render_class()`, boolean attributes use `render_attr()`. All processing happens inline in f-strings with no post-processing step.

---

## Zero-Build Loading

Python calls the Rust compiler through PyO3 and executes generated code in memory. Applications keep only `.hyper` source files.

Compilation returns generated Python plus structured metadata:

```text
file mode
implicit component name
exports
source ranges
expression brace ranges
```

Implicit components are exposed as callable `Component` objects on their containing package:

```python
from app.components import Button
```

Component libraries load as normal modules:

```python
from app.components.controls import Button, Link
```

The loader uses compiler metadata rather than filename capitalization or generated-source inspection. Import order cannot change an implicit component into a module.

Automatic activation installs a lightweight finder without importing the runtime, MarkupSafe, optional integrations, or the native compiler until a `.hyper` file is requested.

---

## Source Maps and IDE Integration

Generation records Python and HTML source ranges. Mapping validates byte offsets before converting source positions to UTF-16 for JetBrains.

The IDE bridge transports generated code, ranges, and diagnostics. It does not expose file generation as a user command or write `.py` files on save.

---

## Security

### Auto-Escaping

All expressions are escaped via direct function calls:

```hyper
<div>{user_input}</div>
```
```python
yield f"""<div>{escape(user_input)}</div>"""
```

`escape()` converts values to strings and HTML-escapes them. Static HTML (no expressions) yields plain strings without processing.

### Raw HTML

Use `safe()` for trusted content:

```hyper
<div>{safe(html_content)}</div>
```

Only use `safe()` on content you control.

### Trusted Source

A `.hyper` file may contain Python. Its generated module code executes on import and its component code executes on render. Treat templates as application source code, not untrusted content.

Control imports through virtual environments, application boundaries, and code review.

---

## Error Contracts

Compiler errors preserve filename, source ranges, related labels, and help text across PyO3. The binding does not reduce them to message-only exceptions.

Generated Python uses a stable synthetic filename registered with `linecache`, so syntax errors and runtime tracebacks do not show `<string>`.

Representative corrections live in the language guide. Exact source spans and formatting live in `.expected.err` tests and follow [Error Messages](../standards/error-messages.md).

---

## Compile-Time Validation

Reject these errors before rendering:

- unclosed tags;
- mismatched tags;
- children in void elements;
- duplicate attributes;
- invalid HTML nesting.

---

**See Also:**
- [Templates Syntax](../design/templates.md) - Language guide
