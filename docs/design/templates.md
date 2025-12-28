# Templates & Layouts

Hyper uses `.hyper` files for HTML templates with Python variables and control flow.

---

# Your First Template

```python
# Greeting.hyper

name: str

<h1>Hello {name}</h1>
```

Put variables in curly braces. Under the hood, these are just Python f-strings.

---

# Simple Layouts

Layouts provide shared structure across pages.

## Create a Layout

Make a file: `app/pages/Layout.hyper`

```python
# app/pages/Layout.hyper

<!doctype html>
<html>
<head>
    <title>My Site</title>
</head>
<body>
    {...}
</body>
</html>
```

Use `{...}` to mark where page content goes.

<details>
<summary><strong>Note:</strong> Alternative <code>slot</code> syntax</summary>

Alternatively, use `slot` syntax instead of `...`:

```python
<body>
    {slot} <!-- or <{slot}/> -->
</body>
```

This guide uses `...`.

</details>

## Use a Layout

```python
# app/pages/index.hyper

from app.pages import Layout

<{Layout}>
    <h1>Welcome Home!</h1>
    <p>This is the homepage.</p>
</{Layout}>
```

Content between `<{Layout}>` and `</{Layout}>` replaces `{...}`.

## Layout Props

```python
# app/pages/Layout.hyper

title: str = "My Site"

<!doctype html>
<html>
<head>
    <title>{title}</title>
</head>
<body>
    {...}
</body>
</html>
```

Pass props:

```python
# app/pages/index.hyper

from app.pages import Layout

<{Layout} title="Home">
    <h1>Welcome!</h1>
</{Layout}>
```

Pass variables:

```python
# app/pages/index.hyper

from datetime import datetime
from app.pages import Layout

title = "Home | " + datetime.now().strftime("%Y")

<{Layout} title={title}>
    <h1>Welcome!</h1>
</{Layout}>
```

## Shorthand Props

Skip the attribute name if the variable matches it.

```python
# app/pages/index.hyper

from app.pages import Layout

title = "Home"

<{Layout} {title}>
    <h1>Welcome!</h1>
</{Layout}>
```

This expands to `title={title}`.

## Default Slot Content

Give slots fallback content:

```python
# app/pages/Layout.hyper

<body>
    <{...}>
        <p>Default content if slot is empty.</p>
    </{...}>
</body>
```

---

# Advanced Layouts

## Named Slots

```python
# app/pages/Layout.hyper

<body>
    <aside>
        <{...} name="sidebar" />
    </aside>
    <main>
        {...}
    </main>
</body>
```

```python
# app/pages/index.hyper

from app.pages import Layout

<{Layout}>
    <{...} name="sidebar">
        <nav>
            <a href="/settings">Settings</a>
        </nav>
    </{...}>
    <h1>Dashboard Content</h1>
</{Layout}>
```

## Nest Layouts

```
app/
  pages/
  layouts/
  components/
```

```python
# app/layouts/BlogLayout.hyper

from app.layouts import Layout

title: str = "Blog"

<{Layout} {title}>
    <div class="blog-container">
        {...}
    </div>
</{Layout}>
```

---

# Components

Put reusable pieces in `app/components`.

## Create a Component

```python
# app/components/Button.hyper

<button type="button" {...}>
    {...}
</button>
```

The `{...}` spreads attributes and content based on context.

## Use a Component

```python
# app/pages/index.hyper

from app.components import Button

<{Button} hx-post="/settings/save" class="bg-white text-black rounded">
    Save
</{Button}>
```

Output:

```html
<button type="button" hx-post="/settings/save" class="bg-white text-black rounded">
  Save
</button>
```

---

# Control Flow

## For Loops

```python
# app/pages/users.hyper

from app.models import User

users: list[User]

<div class="users">
    for user in users:
        <div class="user">{user.name}</div>
    end
</div>
```

---

## Conditionals

```python
# app/pages/nav.hyper

is_admin: bool

<nav>
    if is_admin:
        <a href="/admin">Admin</a>
    else:
        <a href="/account">Account</a>
    end
</nav>
```

---

## Match

Pattern match against values.

```python
# app/components/Status.hyper

status: str

<div>
    match status:
        case "loading":
            <p>Loading...</p>
        case "error":
            <p>Error!</p>
        case "success":
            <p>Done!</p>
    end
</div>
```

Match numeric values:

```python
# app/components/HttpStatus.hyper

code: int

<div>
    match code:
        case 200:
            <span>OK</span>
        case 404:
            <span>Not Found</span>
        case 500:
            <span>Server Error</span>
        case _:
            <span>Unknown Status</span>
    end
</div>
```

---

# Advanced Attributes

## Spread Attributes

Spread attributes from a component to its root element.

```python
# app/components/Button.hyper

<button class="text-black bg-white" {...}>
    {...}
</button>
```

Attributes passed to the component merge with existing ones:

```python
# app/pages/index.hyper

from app.components import Button

disabled = False

<{Button} class="border border-neutral-900" type="button" {disabled}>
    Click Me
</{Button}>
```

Output:

```html
<button class="text-black bg-white border border-neutral-900" type="button">
   Click Me
</button>
```

Classes merge. Other attributes override.

---

## Conditional Classes

Combine class strings, arrays, and objects:

```python
class = [
    "bg-neutral-950 text-neutral-100",
    "border border-neutral-900",
    {"active": False, "disabled": True}
]

<button {class}>Click</button>
```

Output:

```html
<button class="bg-neutral-950 text-neutral-100 border border-neutral-900 disabled">
  Click
</button>
```

---

## Dynamic Styles

Pass style object:

```python
style = {"color": "red", "font-weight": "bold"}

<p {style}>Important</p>
```

Output:

```html
<p style="color:red;font-weight:bold">Important</p>
```

---

## Data Attributes

Single data attribute:

```python
state = "success"

<div data-state={state}>...</div>
```

Output:

```html
<div data-state="success">...</div>
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

<div {data}>...</div>
```

Output:

```html
<div data-user='{"id":"Aaron","role":"admin"}' data-state="success">...</div>
```

---

## Spread Attributes

Spread multiple attributes at once:

```python
attrs = {
    "href": "https://example.com",
    "target": "_blank",
}

<a {**attrs}>Link</a>
```

Output:

```html
<a href="https://example.com" target="_blank">Link</a>
```

---

## Boolean Attributes

`True` renders the attribute. `False` omits it.

```python
<input disabled={True} readonly={False} />
```

Output:

```html
<input disabled>
```

---

# Comments

Nothing special here.

Write HTML comments for client-side comments.

```python
<!-- This appears in page source -->
```

Write Python comments for server-side comments.

```python
# This won't appear in page source
<h1>Title</h1>
```

---

# Escaping & Trusted HTML

Hyper escapes interpolated values by default.

Render trusted HTML with `:safe`:

```python
{post.content:safe}
```

---

# Fragments

Render a section in place AND make it callable independently.

```python
# app/pages/Layout.hyper

user: User

<html>
    <body>
        @fragment
        def Sidebar(user: User):
            <aside>{user.name}</aside>
        end

        <main>{...}</main>
    </body>
</html>
```

Call the fragment standalone:

```python
from app.pages import Layout

Layout.Sidebar(user=current_user)
```

Or import directly:

```python
from app.pages.Layout import Sidebar

Sidebar(user=current_user)
```

---

# Functions

## Component Functions

Functions that contain HTML produce markup. No `return` needed.

```python
def Card(title: str):
    <div class="card">
        <h2>{title}</h2>
        {...}
    </div>
```

## Python Functions

Regular Python functions use `return`.

```python
def format_date(d: datetime) -> str:
    return d.strftime("%B %d, %Y")

<p>Last login: {format_date(user.last_login)}</p>
```

---

# HTML Variables

Store HTML in variables for reuse:

```python
title = <span>Welcome</span>

<div class="card">
    {title}
</div>
```

Multi-line with parentheses:

```python
header = (
    <div class="header">
        <h1>Title</h1>
        <p>Subtitle</p>
    </div>
)
```

## Inline Components with Lambda

```python
greet = lambda name: <span>Hello {name}</span>

<div>
    {greet("John")}
    <{greet} name="Jane"/>
</div>
```

Multi-line lambda:

```python
card = lambda title, subtitle: (
    <div class="card">
        <h1>{title}</h1>
        <p>{subtitle}</p>
    </div>
)

<{card} title="Welcome" subtitle="Get started"/>
```

---

# Multiple Components Per File

```python
# app/components/forms.hyper

def Form(action: str):
    <form {action}>{...}</form>

def Input(name: str, type: str = "text"):
    <input {name} {type}/>

def Button(type: str = "submit"):
    <button {type}>{...}</button>
```

```python
from app.components.forms import Form, Input, Button

<{Form} action="/login">
    <{Input} name="email" type="email"/>
    <{Button}>Sign In</{Button}>
</{Form}>
```

---

**[← Previous: Routing](routing.md)** | **[Next: Fragments →](fragments.md)**