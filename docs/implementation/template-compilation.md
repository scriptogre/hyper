# Template Compilation

Hyper compiles hyper templates to readable Python functions.

---

# Basic Workflow

Mark a package as containing hyper templates:

```python
# components/__init__.py
from hyper_templates import enable_components
```

Write a hyper template:

```python
# components/Button.py
icon: str
text: str

t"""<button>{icon} {text}</button>"""
```

Use it anywhere:

```python
from .components import Button

html = Button(icon="→", text="Click")
```

The import activates compilation automatically. Hyper templates become callable functions.

---

# Conditionals

Hyper templates with `if/else` compile to native Python conditionals.

**You write:**

```python
show_admin: bool

t"""
<!--@ if {show_admin} -->
  <p>Admin view</p>
<!--@ else -->
  <p>User view</p>
<!--@ end -->
"""
```

**Compiles to:**

```python
def render(show_admin: bool) -> str:
    if show_admin:
        return t"<p>Admin view</p>"
    else:
        return t"<p>User view</p>"
```

Only the executed branch runs.

---

# For Loops

Hyper templates with `for` compile to native Python loops.

**You write:**

```python
items: list[Item]

t"""
<ul>
  <!--@ for item in {items} -->
    <li>{item.name}</li>
  <!--@ end -->
</ul>
"""
```

**Compiles to:**

```python
def render(items: list[Item]) -> str:
    _items = []
    for item in items:
        _items.append(t"<li>{item.name}</li>")

    return t"<ul>{''.join(_items)}</ul>"
```

The loop iterates at render time, not at template definition time.

---

# Development Mode

Set the `DEBUG` environment variable to enable hot reload:

```bash
DEBUG=true python app.py
```

Hyper watches for file changes and recompiles hyper templates automatically.

In production, omit `DEBUG` to compile once without watching.

---

# Async Templates

If a hyper template contains `await`, it compiles to an async function automatically.

**You write:**

```python
t"""
<ul>
  <!--@ for user in {await db.get_users()} -->
    <li>{user.name}</li>
  <!--@ end -->
</ul>
"""
```

**Compiles to:**

```python
async def render() -> str:
    _items = []
    for user in await db.get_users():
        _items.append(t"<li>{user.name}</li>")

    return t"<ul>{''.join(_items)}</ul>"
```

Use with `await`:

```python
html = await UserList()
```

Templates without `await` compile to sync functions. No marker needed.

---

# Type Checking

Top-level annotations become function parameters:

```python
icon: str
text: str

t"""<button>{icon} {text}</button>"""
```

Type checkers like `ty` verify the compiled function signatures. Editors provide autocomplete.

---

# Why Python

Hyper template compilation happens once per template and is cached. Python's AST module is fast enough for this.

Jinja2 and Mako prove pure Python works in production at scale.
