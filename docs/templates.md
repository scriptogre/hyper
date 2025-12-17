# Templates & Layouts

Hyper uses Python 3.14 t-strings for HTML templates with Python variables.

---

# Your First Template

```python
name = "Alice"

t"""
<h1>Hello {name}</h1>
"""
```

Put variables in curly braces.

---

# Simple Layouts

Layouts provide shared structure across pages.

## Create a Layout

Make a file: `app/pages/Layout.py`

```python
# app/pages/Layout.py

t"""
<!doctype html>
<html>
<head>
    <title>My Site</title>
</head>
<body>
    {...}
</body>
</html>
"""
```

Use `{...}` to mark where page content goes.

<details>
<summary><strong>Note:</strong> Alternative <code>slot</code> syntax</summary>

Alternatively, use `slot` syntax instead of `...`:

```python
from hyper import slot

t"""
<body>
    {slot} <!-- or <{slot}/> -->
</body>
"""
```

Both are identical, except that `slot` requires an import.

This guide uses `...`.

</details>

## Use a Layout

```python
# app/pages/index.py

from app.pages import Layout

t"""
<{Layout}>
    <h1>Welcome Home!</h1>
    <p>This is the homepage.</p>
</{Layout}>
"""
```

Content between `<{Layout}>` and `</{Layout}>` replaces `{...}`.

## Layout Props

```python
# app/pages/Layout.py

title: str = "My Site"

t"""
<!doctype html>
<html>
<head>
    <title>{title}</title>
</head>
<body>
    {...}
</body>
</html>
"""
```

Pass props:

```python
# app/pages/index.py

from app.pages import Layout

t"""
<{Layout} title="Home">
    <h1>Welcome!</h1>
</{Layout}>
"""
```

Pass variables:

```python
# app/pages/index.py

from datetime import datetime
from app.pages import Layout

title = "Home | " + datetime.now().strftime("%Y")

t"""
<{Layout} title={title}>
    <h1>Welcome!</h1>
</{Layout}>
"""
```

## Shorthand Props

Skip the attribute name if the variable matches it.

```python
# app/pages/index.py

from app.pages import Layout

title = "Home"

t"""
<{Layout} {title}>
    <h1>Welcome!</h1>
</{Layout}>
"""
```

This expands to `title={title}`.

## Default Slot Content

Give slots fallback content:

```python
# app/pages/Layout.py

t"""
<body>
    <{...}>
        <p>Default content if slot is empty.</p>
    </{...}>
</body>
"""
```

---

# Advanced Layouts

## Named Slots

```python
# app/pages/Layout.py

t"""
<body>
    <aside>
        <{...} name="sidebar" />
    </aside>
    <main>
        {...}
    </main>
</body>
"""
```

```python
# app/pages/index.py

from app.pages import Layout

t"""
<{Layout}>
    <{...} name="sidebar">
        <nav>
            <a href="/settings">Settings</a>
        </nav>
    </{...}>
    <h1>Dashboard Content</h1>
</{Layout}>
"""
```

## Nest Layouts

```
app/
  pages/
  layouts/
  components/
```

```python
# app/layouts/BlogLayout.py

from app.layouts import Layout

title: str = t"Blog - {...}"

t"""
<{Layout} {title}>
    <div class="blog-container">
        {...}
    </div>
</{Layout}>
"""
```

### Exploring: inline `...` conditional (not decided)

Potential future idea (more magic, still to explore):

```python
title: str = (t"Blog - {...}" if ... else "Blog")
```

---

# Components

Put reusable pieces in `app/components`.

## Create a Component

```python
# app/components/Button.py

t"""
<button type="button" {...}>
    {...}
</button>
"""
```

The `{...}` spreads attributes and content based on context.

## Use a Component

```python
from app.components import Button

t"""
<{Button} hx-post="/settings/save" class="bg-white text-black rounded">
    Save
</{Button}>
"""

# <button type="button" hx-post="/settings/save" class="bg-white text-black rounded">
#   Save
# </button>
```

---

# Control Flow

## For Loops

Use directive comments for block loops.

```python
from app.models import User

users: list[User]
user: User

t"""
<div class="users">
    <!--@ for {user} in {users} -->
        <div class="user">{user.name}</div>
    <!--@ end -->
</div>
"""
```

Alternative syntax: `{@: for {item} in {items}}`

---

## Conditionals

```python
is_admin: bool

t"""
<nav>
    <!--@ if {is_admin} -->
        <a href="/admin">Admin</a>
    <!--@ else -->
        <a href="/account">Account</a>
    <!--@ end -->
</nav>
"""
```

Alternative syntax: `{@: if {condition}}`

---

# Advanced Attributes

## Spread Attributes

Spread attributes from a component to its root element.

```python
# app/components/Button.py

t"""
<button class="text-black bg-white" {...}>
    {...}
</button>
"""
```

Attributes passed to the component merge with existing ones:

```python
from app.components import Button

disabled = False

t"""
<{Button} class="border border-neutral-900" type="button" {disabled}>
    Click Me
</{Button}>
"""

# <button class="text-black bg-white border border-neutral-900" type="button">
#    Click Me
# </button>
```

Classes merge. Other attributes override.

---

## Conditional Classes

Combine class strings, arrays, and objects:

```python
_class = [
    "bg-neutral-950 text-neutral-100",
    "border border-neutral-900",
    {"active": False, "disabled": True}
]

t'<button class={_class}>Click</button>'

# <button class="bg-neutral-950 text-neutral-100 border border-neutral-900 disabled">
#   Click
# </button>
```

Use `_class` because `class` is a Python keyword.

---

## Dynamic Styles

Pass style object:

```python
style = {"color": "red", "font-weight": "bold"}

t'<p {style}>Important</p>'

# <p style="color:red;font-weight:bold">Important</p>
```

---

## Data Attributes

Single data attribute:

```python
state = "success"

t'<div data-state={state}>...</div>'

# <div data-state="success">...</div>
```

Multiple data attributes:

```python
data = {
    "user": {
        "id": "Aaron",
        "role": "admin",
    },
    "state": "success",
}

t'<div {data}>...</div>'

# <div data-user='{"id":"Aaron","role":"admin"}' data-state="success">...</div>
```

Flatten nested objects:

```python
t'<div {data:flat}>...</div>'

# <div data-user-id="Aaron" data-user-role="admin" data-state="success">...</div>
```

---

## Spread Attributes

Spread multiple attributes at once:

```python
attrs = {
    "href": "https://example.com",
    "target": "_blank",
}

t'<a {attrs}>Link</a>'

# <a href="https://example.com" target="_blank">Link</a>
```

---

## Boolean Attributes

`True` renders the attribute. `False` omits it.

```python
t'<input disabled={True} readonly={False} />'

# <input disabled>
```

---

# Comments

HTML comments are sent to the browser:

```python
t"""
<!-- This appears in page source -->
<h1>Title</h1>
"""
```

Server-side comments are stripped from output:

```python
t"""
<!--# This won't appear in page source -->
<h1>Title</h1>
"""
```

Use directive comments for control flow:

```python
t"""
<!--@ if {show_title} -->
  <h1>Title</h1>
<!--@ end -->
"""
```

---

# Escaping & Trusted HTML

Hyper escapes interpolated values by default.

Render trusted HTML with `:safe`:

```python
t"{post.content:safe}"
```

---

**[← Previous: Routing](routing.md)** | **[Next: Content →](content.md)**
