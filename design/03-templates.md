# Templates & Layouts

Hyper uses Python 3.14's t-string templates for HTML with Python variables.

---

# Your First Template

```python
name = "Alice"

t'''
<h1>Hello {name}</h1>
'''
```

Variables go in curly braces. They're automatically escaped for security.

---

# Layouts

## Create a Layout

Layouts provide common structure across pages.

Make a file: `app/layouts/Base.py`

```python
# app/layouts/Base.py
t'''
<!doctype html>
<html>
<head>
    <title>My Site</title>
</head>
<body>
    {...}
</body>
</html>
'''
```

The `{...}` marks where page content goes.

<details>
<summary><strong>Note:</strong> Alternative <code>slot</code> syntax</summary>

You can use `slot` syntax instead of `...`:

```python
from hyper import slot

t'''
<body>
    {slot} <!-- or <{slot}/> -->
</body>
'''
```

Both are identical, except that `slot` requires an import.

This guide uses `...`.
</details>

## Use a Layout

```python
# app/routes/index.py
from app.layouts import Base

t'''
<{Base}>
    <h1>Welcome Home!</h1>
    <p>This is the homepage.</p>
</{Base}>
'''
```

Content between `<{Base}>` and `</{Base}>` replaces `{...}`.

## Layout Props

Make layouts dynamic with props.

```python
# app/layouts/Base.py

title: str = "My Site"  # Default value

t'''
<!doctype html>
<html>
<head>
    <title>{title}</title>
</head>
<body>
    {...}
</body>
</html>
'''
```

Define props at the top. Use them in the template.

**Pass props as attributes:**

```python
from app.layouts import Base

t'''
<{Base} title="Homepage">
    <h1>Welcome!</h1>
</{Base}>
'''
```

**Or import for editor support:**

```python
from app.layouts import Base
from app.layouts.Base import title

t'''
<{Base} {title}="Homepage">
    <h1>Welcome!</h1>
</{Base}>
'''
```

<details>
<summary><strong>Why use <code>{title}</code> syntax?</strong></summary>

Importing the prop variable gives you:
- Autocomplete in your editor
- Type checking
- Impossible to typo prop names
- Makes it clear which are props vs. regular HTML attributes

```python
from app.layouts import Base
from app.layouts.Base import title, lang  # Import props

t'''
<{Base} {title}="Home" {lang}="en">
    ...
</{Base}>
'''
```

For brevity, you can import all props using `*`:

```python
from app.layouts.Base import *  # Import all props
```

But beware that it may clutter your namespace.

```python
from app.layouts.Base import title, lang
```

</details>

## Default Slot Content

Slots can have fallback content:

```python
t'''
<div class="alert">
    <{...}>
        <p>Default message</p>
    </{...}>
</div>
'''
```

If no content is provided, it shows the default. Otherwise, it's replaced.

## Named Slots

Layouts can have multiple content areas.

```python
# app/layouts/Dashboard.py

t'''
<!doctype html>
<html>
<body>
    <aside>
        <{...} name="sidebar"/>
    </aside>
    
    <main>
        {...}
    </main>
</body>
</html>
'''
```

Fill the slots:
```python
from app.layouts import Dashboard

t'''
<{Dashboard}>
    <{...} name="sidebar">
        <nav>
            <a href="/settings">Settings</a>
        </nav>
    </{...}>

    <h1>Dashboard Content</h1>
    <p>This goes in the unnamed slot.</p>
    
    
    {comment("This won't be rendered")}
    
</{Dashboard}>
'''
```

You can also fill slots using attributes, rather than tag syntax:
```python
from app.layouts import Dashboard

t'''
<{Dashboard}>
    <nav {...}="sidebar">
        <a href="/settings">Settings</a>
    </nav>

    <h1>Dashboard Content</h1>
    <p>This goes in the unnamed slot.</p>
</{Dashboard}>
'''
```

**Note:** Content NOT inside a named slot goes to the unnamed slot (`{...}`).

<details>
<summary><strong>Alternative:</strong> Using <code>{slot}</code></summary>

```python
# Layout
from hyper import slot

t'''
<aside>
    <{slot} name="sidebar"/>
</aside>
<main>
    <{slot}/>
</main>
'''
```

```python
# Usage
from hyper import slot

t'''
<{Dashboard}>
    <{slot} name="sidebar">
        <nav>...</nav>
    </{slot}>
    
    <h1>Main content</h1>
</{Dashboard}>
'''
```

```python
# Or with attribute syntax
from hyper import slot

t'''
<{Dashboard}>
    <nav {slot}="sidebar">...</nav>

    <h1>Main content</h1>
</{Dashboard}>
'''
```

</details>

## Nest Layouts

```python
# layouts/BlogLayout.py
from app.layouts import Base

blog_title: str

t'''
<{Base} title={f"Blog - {blog_title}"}>
    <div class="blog-container">
        {...}
    </div>
</{Base}>
'''
```

---

# Components

## Create a Component

Components are reusable pieces in `components/`.

Make a file: `components/Card.py`

```python
# components/Card.py
title: str

t'''
<div class="bg-white shadow rounded p-4">
    <h3>{title}</h3>
    <div class="flex flex-col gap-2">
        {...}
    </div>
</div>
'''
```

Use it:

```python
from components import Card

t'''
<{Card} title="User Info">
    <p>Name: Alice</p>
    <p>Email: alice@example.com</p>
</{Card}>
'''
```

Self-closing syntax for components without content:
```python
from components import Button

t'''
<{Button} text="Click Me" />
'''
```

---

## For Loops

Use list comprehensions to render lists of components or HTML:

```python
users = User.all()

t'''
<div class="users">
    {[t'<div class="user">{user.name}</div>' for user in users]}
</div>
'''
```

With components:
```python
from components import UserCard

t'''
<div class="users">
    {[t'<{UserCard} user={user}/>' for user in users]}
</div>
'''
```

---

## Safe HTML

Variables are auto-escaped. To render trusted HTML:
```python
# Option 1: Format specifier
t'{post.html_content:safe}'

# Option 2: Markup class
from hyper import Markup
safe_content = Markup(post.html_content)
t'{safe_content}'
```

---

## Advanced Attributes

### Conditional Classes
```python
classes = ["btn", {"active": is_active, "disabled": is_disabled}]
t'<button class={classes}>Click</button>'
```

### Dynamic Styles
```python
styles = {"color": "red", "font-weight": "bold"}
t'<p style={styles}>Important</p>'
```

### Data Attributes
```python
data = {"user-id": 123, "role": "admin"}
t'<div data={data}>Content</div>'
# <div data-user-id="123" data-role="admin">
```

### Spread Attributes
```python
attrs = {"href": "https://example.com", "target": "_blank"}
t'<a {attrs}>Link</a>'
```

### Boolean Attributes
```python
t'<input disabled={True} readonly={False} />'
# <input disabled>
```

---
```markdown
## Comments

**HTML comments** (sent to browser):
```python
t'''
<!-- This appears in page source -->
<h1>Title</h1>
'''
```

**Server-side comments** (stripped from output):
```python
t'''
<!--# This won't appear in page source #-->
<h1>Title</h1>
'''
```

**Multi-line server comments:**
```python
t'''
<!--#
This is a multi-line comment
that won't be sent to the browser
#-->
<h1>Title</h1>
'''
```

---

## Fragments

Fragments let you render just part of a page. Perfect for HTMX partial updates.

### Basic Usage

Mark an element as a fragment with `{fragment}`:

```python
from hyper import fragment

users = User.all()

t'''
<div>
    <h1>Users</h1>

    <div id="user-list" {fragment}>
        <ul>
            {[t'<li>{u.name}</li>' for u in users]}
        </ul>
    </div>
</div>
'''
```

By default, `{fragment}` uses the element's `id` as the fragment name.

**If the element has no `id`, it will fail.**

### Custom Fragment Names

Override the fragment name:

```python
from hyper import fragment

user = get_user(user_id)

t'''
<div>
    <h1>{user.name}'s Profile</h1>

    <div {fragment}="profile-info">
        <p>Email: {user.email}</p>
        <p>Joined: {user.created_at}</p>
    </div>
</div>
'''
```

Fragment names can be dynamic:

```python
t'''
<form {fragment}="edit-user-{user.id}" method="POST">
    <input name="email" value="{user.email}">
    <button>Update</button>
</form>
'''
```

### Rendering Fragments

Control which fragments render from your route:

```python
from typing import Annotated
from hyper import Request, Header, render, fragment

request: Request
is_htmx: Annotated[str | None, Header("HX-Request")]

users = User.all()

t'''
<div>
    <h1>Users</h1>
    <div id="user-list" {fragment}>
        <ul>{[t'<li>{user.name}</li>' for user in users]}</ul>
    </div>
</div>
'''

if is_htmx:
    render(fragments=["user-list"])  # Render only this fragment
```

The framework automatically calls `render()` at the end if you don't. Call `render(fragments=[...])` to extract specific fragments instead of the full page.

### Multiple Fragments

```python
from typing import Annotated
from hyper import Query, fragment, render

users = User.all()
stats = get_stats()

t'''
<div>
    <div id="user-list" {fragment}>
        <h2>Users</h2>
        <ul>{[t'<li>{u.name}</li>' for u in users]}</ul>
    </div>

    <div id="stats" {fragment}>
        <h2>Statistics</h2>
        <p>Total: {stats.total}</p>
    </div>
</div>
'''

fragments: Annotated[list, Query("_fragment")] = []

if fragments:
    render(fragments=fragments)
```

Request `/?_fragment=user-list&_fragment=stats` to render both fragments.

### Streaming with Fragments

```python
from hyper import render, fragment

user = User.get(id=user_id)

t'''
<div>
    <h1>{user.name}</h1>
    <div id="status" {fragment}>
        Status: {user.status}
        Last updated: {user.updated_at}
    </div>
</div>
'''

async def stream():
    yield render(fragments=["status"])

    async for update in subscribe_to_updates(user_id):
        yield render(fragments=["status"])
```

The template logic re-runs on each iteration, keeping fragments in sync with current data.

---

## Whitespace Control

Hyper automatically removes unwanted whitespace from your templates so you can write readable, indented code that produces clean HTML.

### Global Settings (Enabled by Default)

Two settings control how whitespace is handled:

**`trim_newlines`** - Removes the first newline after expressions and list comprehensions.

**`trim_indent`** - Removes leading spaces/tabs from lines containing expressions.

Both are **enabled by default**. You can configure them in `main.py` if needed:

```python
# main.py
from hyper import Hyper

app = Hyper(
    templates={
        "trim_newlines": True,   # Default
        "trim_indent": True,     # Default
    }
)
```

### How It Works

**With both enabled (default):**

```python
users = ["Alice", "Bob"]

t'''
<div>
    <ul>
        {[t'<li>{user}</li>' for user in users]}
    </ul>
</div>
'''
```

Renders as:
```html
<div>
    <ul>
        <li>Alice</li><li>Bob</li>
    </ul>
</div>
```

**With `trim_newlines=False`:**

```python
t'''
<div>
    <ul>
        {[t'<li>{user}</li>' for user in users]}
    </ul>
</div>
'''
```

Renders as:
```html
<div>
    <ul>

        <li>Alice</li><li>Bob</li>
    </ul>
</div>
```
(Extra blank line after `<ul>`)

**With `trim_indent=False`:**

```python
t'''
<div>
    <ul>
        {[t'<li>{user}</li>' for user in users]}
    </ul>
</div>
'''
```

Renders as:
```html
<div>
    <ul>
<li>Alice</li><li>Bob</li>
    </ul>
</div>
```
(Indentation before expression is preserved, breaking alignment)

### Automatic Exceptions

Trimming is **automatically disabled** inside elements where whitespace matters:

- `<pre>` - Preformatted text
- `<code>` - Code blocks
- `<textarea>` - Form inputs
- `<script>` - JavaScript (whitespace can affect code)
- `<style>` - CSS (whitespace can affect selectors)

```python
t'''
<pre>
    {code_snippet}
</pre>
'''
```

Whitespace is preserved inside `<pre>`, so indentation remains intact.

### Per-Element Override with `{trim}`

Override trimming behavior for specific elements:

```python
from hyper import trim

# Override settings for this element's children
t'''<div {trim}={{"newlines": False, "indent": True}}>{content}</div>'''

# Force trimming inside <pre> (override automatic exception)
t'''<pre {trim}={{"newlines": True, "indent": True}}>{content}</pre>'''

# Strip ALL whitespace inside element
t'''<div {trim}="all">{content}</div>'''
```

### The `:empty` Selector Solution

Use `{trim}="all"` for truly empty elements:

```python
from hyper import trim

user_message: str = ""

t'''
<div class="empty:hidden" {trim}="all">
    {user_message}
</div>
'''
```

When `user_message` is empty, renders as:
```html
<div class="empty:hidden"></div>
```

The div is truly empty, so the `:empty` CSS selector works correctly.

**Without `{trim}="all"`**, the div would contain whitespace from the template:
```html
<div class="empty:hidden">

</div>
```

The `:empty` selector would not match.

---

## Migrating from Jinja2

### Template Syntax

| Jinja2                                 | Hyper                                                      |
|----------------------------------------|------------------------------------------------------------|
| `{% extends "base.html" %}`            | `from app.layouts import Base` then `<{Base}>...</{Base}>` |
| `{% block content %}...{% endblock %}` | `{...}` or `<{...} name="content"/>`                       |
| `{% include "header.html" %}`          | `from components import Header` then `<{Header}/>`         |
| `{% if condition %}`                   | `{condition and t'<div>...</div>'}`                        |
| `{% for item in items %}`              | `{[t'<div>{item}</div>' for item in items]}`               |
| `{{ variable }}`                       | `{variable}`                                               |
| `{{ variable\|safe }}`                 | `{variable:safe}`                                          |
| `{# comment #}`                        | `<!--# comment #-->`                                       |
| `{% set x = value %}`                  | `x = value` (at module top)                                |

### Filters

| Jinja2                               | Hyper                               |
|--------------------------------------|-------------------------------------|
| `{{ value\|upper }}`                 | `{value.upper()}`                   |
| `{{ value\|lower }}`                 | `{value.lower()}`                   |
| `{{ value\|title }}`                 | `{value.title()}`                   |
| `{{ value\|capitalize }}`            | `{value.capitalize()}`              |
| `{{ value\|length }}`                | `{len(value)}`                      |
| `{{ value\|default("N/A") }}`        | `{value:default("N/A")}`            |
| `{{ value\|trim }}`                  | `{value.strip()}`                   |
| `{{ value\|join(", ") }}`            | `{", ".join(value)}`                |
| `{{ value\|first }}`                 | `{value[0]}`                        |
| `{{ value\|last }}`                  | `{value[-1]}`                       |
| `{{ value\|sort }}`                  | `{sorted(value)}`                   |
| `{{ value\|reverse }}`               | `{list(reversed(value))}`           |
| `{{ value\|abs }}`                   | `{abs(value)}`                      |
| `{{ value\|int }}`                   | `{int(value)}`                      |
| `{{ value\|float }}`                 | `{float(value)}`                    |
| `{{ value\|round(2) }}`              | `{round(value, 2)}`                 |
| `{{ items\|map(attribute='name') }}` | `{[item.name for item in items]}`   |
| `{{ items\|select }}`                | `{[x for x in items if condition]}` |

---

**[← Previous: Routing](02-routing.md)** | **[Back to Index](README.md)** | **[Next: Dependency Injection →](05-dependency-injection.md)**