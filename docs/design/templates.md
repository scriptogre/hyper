# Templates

Hyper templates are `.hyper` files that combine Python and HTML.

---

## Your First Template

Set up your project:

```
app/
├── components/
│   ├── __init__.py
│   └── Hello.hyper
└── main.py
```

Create `components/Hello.hyper`:

```hyper
<h1>Hello World</h1>
```

Render it in `main.py`:

```python
from components import Hello

print(Hello())
```

```html
<h1>Hello World</h1>
```

<details>
<summary>How imports work</summary>

The compiler automatically updates `__init__.py` to export your templates:

```python
# components/__init__.py (auto-generated)
from .Hello import Hello

__all__ = ["Hello"]
```

This enables clean imports like `from components import Hello`.

To opt out, add `# hyper: no-init` at the top of your `.hyper` file.

</details>

---

## Props

Props make your template accept inputs. Define them above `---`.

Create `components/Greeting.hyper`:

```hyper
name: str

---

<h1>Hello {name}</h1>
```

```python
from components import Greeting

print(Greeting(name="Alice"))
```

```html
<h1>Hello Alice</h1>
```

Props can have defaults:

Create `components/Counter.hyper`:

```hyper
name: str = "World"
count: int = 0

---

<h1>Hello {name}</h1>
<p>Count: {count}</p>
```

```python
from components import Counter

print(Counter())                    # Uses defaults
print(Counter(name="Bob", count=5)) # Override defaults
```

---

## Slots

Slots let your template accept content from the caller.

Add a `layouts/` folder:

```
app/
├── components/
│   └── ...
├── layouts/
│   ├── __init__.py
│   └── Layout.hyper
└── main.py
```

Create `layouts/Layout.hyper`:

```hyper
title: str = "My Site"

---

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

The `{...}` marks where caller content is inserted.

Create `pages/Home.hyper`:

```hyper
from layouts import Layout

---

<{Layout} title="Home">
    <h1>Welcome!</h1>
    <p>This replaces the {...} in Layout.</p>
</{Layout}>
```

Render it:

```python
from pages import Home

print(Home())
```

```html
<!doctype html>
<html>
<head>
    <title>Home</title>
</head>
<body>
    <h1>Welcome!</h1>
    <p>This replaces the {...} in Layout.</p>
</body>
</html>
```

### Default Slot Content

Provide fallback content when nothing is passed:

```hyper
title: str = "My Site"

---

<!doctype html>
<html>
<head>
    <title>{title}</title>
</head>
<body>
    <{...}>
        <p>No content provided.</p>
    </{...}>
</body>
</html>
```

---

## Control Flow

### Conditionals

Create `components/Nav.hyper`:

```hyper
is_admin: bool

---

<nav>
    if is_admin:
        <a href="/admin">Admin</a>
    else:
        <a href="/account">Account</a>
    end
</nav>
```

```python
from components import Nav

print(Nav(is_admin=True))   # <nav><a href="/admin">Admin</a></nav>
print(Nav(is_admin=False))  # <nav><a href="/account">Account</a></nav>
```

### Loops

Create `components/List.hyper`:

```hyper
items: list[str]

---

<ul>
    for item in items:
        <li>{item}</li>
    end
</ul>
```

```python
from components import List

print(List(items=["Apple", "Banana", "Cherry"]))
```

```html
<ul>
    <li>Apple</li>
    <li>Banana</li>
    <li>Cherry</li>
</ul>
```

### Pattern Matching

Create `components/Status.hyper`:

```hyper
status: str

---

match status:
    case "loading":
        <p>Loading...</p>
    case "error":
        <p>Error!</p>
    case _:
        <p>Ready</p>
end
```

```python
from components import Status

print(Status(status="loading"))  # <p>Loading...</p>
print(Status(status="done"))     # <p>Ready</p>
```

---

## Expressions

Use Python expressions directly inside `{}` for inline logic.

### List Comprehensions

> **⚠️ Not Yet Implemented**: HTML inside comprehensions requires a Python expression parser to detect and transform elements into f-strings. Use `for` loops as a workaround.

Generate elements inline without a `for` block:

```hyper
items: list[str]

---

<ul>
    {[<li>{item}</li> for item in items]}
</ul>
```

```python
print(Template(items=["Apple", "Banana"]))
```

```html
<ul>
    <li>Apple</li>
    <li>Banana</li>
</ul>
```

Comprehensions work with any expression:

```hyper
users: list[User]

---

<select>
    {[<option value={u.id}>{u.name}</option> for u in users]}
</select>
```

### Conditional Expressions

Use Python's ternary syntax for inline conditionals:

```hyper
count: int

---

<span>{count} {"item" if count == 1 else "items"}</span>
```

```python
print(Template(count=1))  # <span>1 item</span>
print(Template(count=5))  # <span>5 items</span>
```

Render different elements:

```hyper
is_admin: bool

---

<div>
    {<span class="badge">Admin</span> if is_admin else <span>User</span>}
</div>
```

**Shorthand for optional content**: Omit `else` when you want nothing:

```hyper
show_badge: bool

---

<div>
    {<span class="badge">New</span> if show_badge}
</div>
```

```python
print(Template(show_badge=True))   # <div><span class="badge">New</span></div>
print(Template(show_badge=False))  # <div></div>
```

The compiler transforms `{x if cond}` to `{x if cond else ''}`.

### Short-Circuit Evaluation

Use `and` to conditionally render content:

```hyper
show_warning: bool
message: str

---

<div>
    {show_warning and <p class="warning">{message}</p>}
</div>
```

```python
print(Template(show_warning=True, message="Error!"))
# <div><p class="warning">Error!</p></div>

print(Template(show_warning=False, message="Error!"))
# <div></div>
```

Use `or` for fallback values:

```hyper
title: str | None

---

<h1>{title or "Untitled"}</h1>
```

```python
print(Template(title="Hello"))  # <h1>Hello</h1>
print(Template(title=None))     # <h1>Untitled</h1>
```

### When to Use What

| Pattern | Use Case |
|---------|----------|
| `for...end` block | Multiple elements, complex logic |
| `{[... for ...]}` | Simple inline iteration |
| `if...end` block | Multiple elements, else/elif branches |
| `{x if cond else y}` | Inline choice between two values |
| `{cond and x}` | Conditionally show one thing |
| `{x or fallback}` | Provide default for falsy values |

---

## Named Slots

Templates can have multiple insertion points.

Create `layouts/Layout.hyper`:

```hyper
<!doctype html>
<html>
<body>
    <aside>
        <{...sidebar}>
            <p>Default sidebar</p>
        </{...sidebar}>
    </aside>
    <main>
        {...}
    </main>
</body>
</html>
```

Create `pages/Dashboard.hyper`:

```hyper
from layouts import Layout

---

<{Layout}>
    <{...sidebar}>
        <nav>
            <a href="/">Home</a>
            <a href="/about">About</a>
        </nav>
    </{...sidebar}>

    <h1>Main Content</h1>
    <p>This goes to the default slot.</p>
</{Layout}>
```

```python
from pages import Dashboard

print(Dashboard())
```

```html
<!doctype html>
<html>
<body>
    <aside>
        <nav>
            <a href="/">Home</a>
            <a href="/about">About</a>
        </nav>
    </aside>
    <main>
        <h1>Main Content</h1>
        <p>This goes to the default slot.</p>
    </main>
</body>
</html>
```

### Single-Element Shorthand

When a named slot contains one element, mark it directly:

```hyper
from layouts import Layout

---

<{Layout}>
    <nav {...sidebar}>
        <a href="/">Home</a>
        <a href="/about">About</a>
    </nav>

    <h1>Main Content</h1>
</{Layout}>
```

The `{...sidebar}` on the element marks it as sidebar content. Same output as above.

---

## Attributes

### Dynamic Values

Use expressions directly in attribute values:

```hyper
url: str

---

<a href="{url}">Visit</a>
```

```python
print(Template(url="https://example.com"))
```

```html
<a href="https://example.com">Visit</a>
```

Without quotes also works:

```hyper
element_id: str

---

<button id={element_id}>Click</button>
```

```html
<button id="my-button">Click</button>
```

Multiple substitutions in one attribute:

```hyper
first: str
last: str

---

<button data-name="{first} {last}">Click</button>
```

```html
<button data-name="Alice Smith">Click</button>
```

### Boolean Attributes

`True` renders the attribute. `False` omits it:

```hyper
<button disabled={True} hidden={False}>Submit</button>
```

```html
<button disabled>Submit</button>
```

### Shorthand

When variable name matches attribute name, use shorthand:

```hyper
disabled: bool = False
title: str

---

<button {disabled} {title}>Click</button>
```

The `{disabled}` expands to `disabled={disabled}`.

```python
print(Template(title="Save", disabled=True))
```

```html
<button disabled title="Save">Click</button>
```

**Reserved keywords**: Use `{class}` and `{type}` directly. The compiler handles them:

```hyper
class: list

---

<button {class}>Click</button>
```

Compiles to valid Python using `_class` internally.

### The class Attribute

The `class` attribute has special handling. Provide a list:

```hyper
class = ["btn", "btn-primary", "active"]

---

<button {class}>Click</button>
```

```html
<button class="btn btn-primary active">Click</button>
```

Mix strings and conditional dicts:

```hyper
is_active: bool
is_disabled: bool

---

class = [
    "btn",
    "btn-primary",
    {"active": is_active, "disabled": is_disabled}
]

<button {class}>Click</button>
```

```python
print(Template(is_active=True, is_disabled=False))
```

```html
<button class="btn btn-primary active">Click</button>
```

Falsy values are filtered out:

```hyper
class = ["btn", None, False and "hidden", {"active": True}]

---

<button {class}>Click</button>
```

```html
<button class="btn active">Click</button>
```

### The style Attribute

Provide a dictionary for inline styles:

```hyper
style = {"color": "red", "font-weight": "bold", "margin": "10px"}

---

<p {style}>Important text</p>
```

```html
<p style="color: red; font-weight: bold; margin: 10px">Important text</p>
```

### The data and aria Attributes

Dictionaries expand to prefixed attributes:

```hyper
data = {"user-id": 123, "role": "admin"}
aria = {"label": "Close dialog", "hidden": True}

---

<div {data} {aria}>Content</div>
```

```html
<div data-user-id="123" data-role="admin" aria-label="Close dialog" aria-hidden="true">Content</div>
```

Boolean values in `aria` become `"true"` or `"false"` per ARIA spec.

### Spreading Attributes

Pass a dictionary to spread all its keys as attributes:

```hyper
attrs = {"href": "https://example.com", "target": "_blank"}

---

<a {attrs}>External link</a>
```

```html
<a href="https://example.com" target="_blank">External link</a>
```

Combine spreading with individual attributes:

```hyper
base_attrs = {"id": "my-link"}
target: str = "_blank"

---

<a {base_attrs} {target}>Link</a>
```

```html
<a id="my-link" target="_blank">Link</a>
```

Special attributes like `class` work when spread:

```hyper
class = ["btn", {"active": True}]
attrs = {"class": class, "id": "act_now", "data": {"wow": "such-attr"}}

---

<button {attrs}>Click</button>
```

```html
<button class="btn active" id="act_now" data-wow="such-attr">Click</button>
```

### Capturing Extra Attributes

Accept arbitrary attributes with `**kwargs` syntax:

```hyper
label: str
type: str = "button"
**attrs: dict

---

<button {type} {attrs}>
    {label}
</button>
```

```hyper
from components import Button

---

<{Button} label="Save" class="btn" disabled hx-post="/save" />
```

```html
<button type="button" class="btn" disabled hx-post="/save">
    Save
</button>
```

Use any name: `**attrs`, `**props`, `**extra` all work.

---

## Fragments

Fragments are named sections that render inline AND are importable standalone. Useful for partial updates (e.g., HTMX).

Create `pages/Profile.hyper`:

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

The compiler analyzes which variables each fragment uses.

Render the full page:

```python
from pages import Profile

user = User(name="Alice", bio="Developer")
posts = [Post(title="Hello"), Post(title="World")]

print(Profile(user=user, posts=posts))
```

```html
<div class="page">
    <aside>
        <h3>Alice</h3>
        <p>Developer</p>
    </aside>
    <main>
        <article>Hello</article>
        <article>World</article>
    </main>
</div>
```

Render just the sidebar (without fetching posts):

```python
from pages.Profile import Sidebar  # Import fragment directly

user = User(name="Alice", bio="Developer")

print(Sidebar(user=user))
```

```html
<aside>
    <h3>Alice</h3>
    <p>Developer</p>
</aside>
```

---

## Defining Components

Define reusable components with `def` in the header zone.

Create `components/Cards.hyper`:

```hyper
def Card(title: str):
    <div class="card">
        <h2>{title}</h2>
        {...}
    </div>
end

---

<{Card} title="Welcome">
    <p>Card content here.</p>
</{Card}>

<{Card} title="Another">
    <p>More content.</p>
</{Card}>
```

Use in templates with `<{Name}>` syntax:

```python
from components import Cards

print(Cards())
```

```html
<div class="card">
    <h2>Welcome</h2>
    <p>Card content here.</p>
</div>
<div class="card">
    <h2>Another</h2>
    <p>More content.</p>
</div>
```

Import components directly from Python:

```python
from components.Cards import Card

print(Card("Welcome", slot="<p>Card content.</p>"))
```

```html
<div class="card">
    <h2>Welcome</h2>
    <p>Card content.</p>
</div>
```

Functions and classes in the header are also importable:

```python
from components.Article import format_date
from datetime import datetime

print(format_date(datetime(2024, 12, 25)))
```

```
December 25, 2024
```

---

## Multiple Components Per File

Files without top-level HTML are component libraries.

Create `components/forms.hyper`:

```hyper
def Form(action: str):
    <form {action}>
        {...}
    </form>
end

def Input(name: str, type: str = "text"):
    <input {name} {type} />
end

def Button(type: str = "submit"):
    <button {type}>
        {...}
    </button>
end
```

Create `pages/Login.hyper`:

```hyper
from components.forms import Form, Input, Button

---

<{Form} action="/login">
    <{Input} name="email" type="email" />
    <{Input} name="password" type="password" />
    <{Button}>Sign In</{Button}>
</{Form}>
```

```python
from pages import Login

print(Login())
```

```html
<form action="/login">
    <input name="email" type="email" />
    <input name="password" type="password" />
    <button type="submit">Sign In</button>
</form>
```

---

## Imports and Helpers

Import Python modules and define helpers above `---`.

Create `components/Article.hyper`:

```hyper
from datetime import datetime

def format_date(d: datetime) -> str:
    return d.strftime("%B %d, %Y")

title: str
created_at: datetime

---

<article>
    <h1>{title}</h1>
    <p>Published: {format_date(created_at)}</p>
</article>
```

```python
from datetime import datetime
from components import Article

print(Article(title="Hello", created_at=datetime(2024, 12, 25)))
```

```html
<article>
    <h1>Hello</h1>
    <p>Published: December 25, 2024</p>
</article>
```

---

## Comments

Python comments for server-side (not in output):

```hyper
# This won't appear in HTML
<h1>Title</h1>
```

HTML comments for client-side:

```hyper
<!-- This appears in page source -->
<h1>Title</h1>
```

---

## Escaping

All values are HTML-escaped by default:

```hyper
user_input: str

---

<div>{user_input}</div>
```

If `user_input` is `<script>alert('xss')</script>`, output is:

```html
<div>&lt;script&gt;alert('xss')&lt;/script&gt;</div>
```

Render trusted HTML with `safe()`:

```hyper
html_content: str

---

<div>{safe(html_content)}</div>
```

Only use `safe()` for content you trust (e.g., sanitized HTML from your database).

---

## Streaming

Templates compile to generator functions using `yield`, enabling HTTP streaming.

```python
from components import Feed
from fastapi.responses import StreamingResponse

@app.get("/feed")
def feed():
    return StreamingResponse(
        Feed(posts=all_posts),
        media_type="text/html"
    )
```

The template stays the same:

```hyper
posts: list[Post]

---

<div class="feed">
    for post in posts:
        <article>{post.title}</article>
    end
</div>
```

Each iteration yields a chunk. Use `str(Template())` for buffered output, or iterate for streaming.

---

## File Structure

A `.hyper` file has two zones separated by `---`:

```hyper
# Header zone: imports, defs, props (runs once at import)

from utils import helper

def Badge(text: str):
    <span class="badge">{text}</span>
end

name: str
count: int = 0

---

# Body zone: template code (runs every render)

greeting = f"Hello {name}"

<div>
    <{Badge} text={greeting} />
    <span>{count}</span>
</div>
```

**Header zone** (above `---`):
- `import` statements
- `def` functions — with HTML becomes `@component`, without HTML becomes regular function
- `class` definitions (including `@dataclass`, `Enum`, `Protocol`)
- `NAME: Final[type] = expr` — module-level constants
- `type Name = type_expr` — type aliases (Python 3.12+)
- Type-annotated variables become props: `name: str`, `count: int = 0`
- `**attrs: dict` captures extra attributes

**Body zone** (below `---`):
- Local variables (can reference props)
- HTML template
- Control flow
- `def` functions — closures that can reference props, NOT exported

### When `---` is Required

`---` is required when the header contains **parameters** (props).

| File type | Has parameters? | Needs `---`? |
|-----------|-----------------|--------------|
| HTML only | No | No |
| Parameters + HTML | Yes | **Yes** |
| Library (imports, defs, constants) | No | No |

Simple HTML needs no separator:

```hyper
<div>Hello World</div>
```

Props require the separator:

```hyper
name: str

---

<div>Hello {name}</div>
```

Library files (no top-level HTML) need no separator:

```hyper
def Header(title: str):
    <header>{title}</header>
end

def Footer():
    <footer>Copyright 2024</footer>
end
```
