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
from app.components import Hello

print(Hello())
```

```html
<h1>Hello World</h1>
```

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
from app.components import Greeting
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
from app.components import Counter
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

The `{...}` marks where caller content is inserted. It adds the reserved, keyword-only `content` argument:

```python
from inspect import signature

signature(Layout)
# (*, title: str = "My Site", content=None)
```

Create `pages/Home.hyper`:

```hyper
from app.layouts import Layout
---

<{Layout} title="Home">
    <h1>Welcome!</h1>
    <p>This replaces the {...} in Layout.</p>
</{Layout}>
```

Render it:

```python
from app.pages import Home
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
from app.components import Nav
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
from app.components import List
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
from app.components import Status
print(Status(status="loading"))  # <p>Loading...</p>
print(Status(status="done"))     # <p>Ready</p>
```

### Block Boundaries

Indentation defines Python blocks. Use `end` only when it helps show the boundary:

```hyper
if show_list:
    <ul>
        for item in items:
            <li>{item}</li>
        end
    </ul>
end
```

Both `end` lines above are optional. If present, each must align with its opener. HTML still needs closing tags.

The same rule applies before and after `---`, and in component libraries. Branch clauses share their parent's optional `end`.

Python's single-line form also works without `end`:

```hyper
if missing: return
if ready: <span>Ready</span>
```

---

## Expressions

Use Python expressions directly inside `{}` for inline logic.

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

### Fallback Values

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
| `if...end` block | Multiple elements, else/elif branches |
| `{x if cond else y}` | Inline choice between two values |
| `{x or fallback}` | Provide default for falsy values |

---

## Named Slots

Templates can have multiple insertion points. Each named slot adds a keyword-only argument with the same name.

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
from app.layouts import Layout
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
from app.pages import Dashboard
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
from app.layouts import Layout
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

Default and named slots use the same arguments from Python:

```python
Layout(
    title="Dashboard",
    content=main_content,
    sidebar=navigation,
).render()
```

`content` is reserved for the default slot. A prop cannot share a named slot's name:

```hyper
from hyper import Component

def Panel(*, sidebar: str) -> Component:
    <aside>{...sidebar}</aside>
end
```

```text
error: `sidebar` is both a prop and a named slot

Rename the prop or the slot.
```

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

### Split Attributes Across Lines

Split HTML and component attributes across lines:

```hyper
<div
    id="profile"
    class={["card", {"active": active}]}
    {**attrs}
>
    Content
</div>

<{Card}
    title={title}
    selected
/>
```

The formatting whitespace does not render:

```html
<div id="profile" class="card active" data-role="admin">Content</div>
```

Whitespace inside a quoted value remains part of that value:

```hyper
<button
    _="
        on click
            toggle .active
    "
>
    Toggle
</button>
```

```html
<button _="
        on click
            toggle .active
    ">Toggle</button>
```

Multiline expressions follow Python bracket rules:

```hyper
<div
    class={
        [
            "card",
            {"selected": selected},
        ]
    }
>
    Content
</div>
```

### Boolean Attributes

`True` renders the attribute. `False` omits it:

```hyper
<button disabled={True} hidden={False}>Submit</button>
```

```html
<button disabled>Submit</button>
```

The compiler knows which HTML attributes are boolean (`disabled`, `checked`, `readonly`, `required`, `hidden`, etc.) and handles them automatically. You don't need to think about whether to render `disabled` vs `disabled="true"`. Pass a bool and the compiler does the right thing.

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

Boolean values in `aria` become `"true"` or `"false"` per the [ARIA spec](https://www.w3.org/TR/wai-aria-1.2/#valuetype_true-false). Unlike boolean HTML attributes, ARIA attributes are never omitted. `aria-hidden="false"` and the absence of `aria-hidden` mean different things.

### Spreading Attributes

Use `{**dict}` to spread a dictionary as individual attributes:

```hyper
attrs = {"href": "https://example.com", "target": "_blank"}

---

<a {**attrs}>External link</a>
```

```html
<a href="https://example.com" target="_blank">External link</a>
```

Combine spreading with individual attributes:

```hyper
base_attrs = {"id": "my-link"}
target: str = "_blank"

---

<a {**base_attrs} {target}>Link</a>
```

```html
<a id="my-link" target="_blank">Link</a>
```

Special attributes like `class` work when spread:

```hyper
class = ["btn", {"active": True}]
attrs = {"class": class, "id": "act_now", "data": {"wow": "such-attr"}}

---

<button {**attrs}>Click</button>
```

```html
<button class="btn active" id="act_now" data-wow="such-attr">Click</button>
```

### Pass-Through Attributes

Think of elements as function calls: `<button {**attrs}>` is like calling `button(**attrs)`. The `**` in the parameter declaration collects extra keyword arguments into a dict, and the `**` on the element spreads them back out, the same symmetry as Python functions.

Use one of `{**kwargs}`, `{**props}`, `{**rest}`, `{**attrs}`, or `{**attributes}` to accept and forward arbitrary attributes. The compiler adds it to the function signature automatically.

```hyper
label: str
type: str = "button"

---

<button {type} {**attrs}>
    {label}
</button>
```

```python
def Button(*, label: str, type: str = "button", **attrs):
    ...
```

```hyper
<{Button} label="Save" hx-post="/save" class="btn" disabled />
```

```html
<button type="button" hx-post="/save" class="btn" disabled>
    Save
</button>
```

Other names are not auto-injected:

```hyper
my_dict = {"class": "card"}

<{Card} {**my_dict} />
# my_dict is a local variable, not a parameter
```

---

## Transparent Fragments

Group elements without adding a wrapper:

```hyper
<>
    <button>Save</button>
    <button>Cancel</button>
</>
```

Render text without adding a wrapper:

```hyper
<>Text without a wrapper</>
```

---

## Defining Components

### Annotated Definitions

Use `-> Component` to write a template function. Hyper reads the annotation during compilation, much like FastAPI uses annotations to control behavior.

A standalone `{name}` in that function is template output, not a Python set expression.

```hyper
from hyper import Component

def Heading(*, text: str) -> Component:
    <h1>{text}</h1>
end
```

Calling a template definition creates a lazy component instance. Render it explicitly:

```python
heading = Heading(text="Hello")
html = heading.render()
```

A function without template output remains an ordinary Python factory:

```hyper
from hyper import Component

def Greeting(*, name: str) -> Component:
    return Heading(text=f"Hello, {name}")
end
```

Use bare `return` to stop template output:

```hyper
from hyper import Component

def Greeting(*, name: str | None) -> Component:
    if name is None:
        <p>Hello, stranger</p>
        return
    end
    <h1>Hello, {name}</h1>
end
```

Do not mix template output and `return value` in the same function. Compose components with tags instead:

```hyper
from hyper import Component

def Greeting() -> Component:
    <h1>Hello</h1>
    <{Heading} text="Goodbye" />
end
```

Each function has its own scope. Markup in a nested definition does not make its enclosing factory a template. Ordinary nested helpers retain Python `return` and `yield` behavior.

These examples keep optional `end` markers for visual clarity. See [Block Boundaries](#block-boundaries).

### Export a Subcomponent

Nested component definitions use normal Python scope. They may capture values from their enclosing function:

```hyper
from hyper import Component

def Greeting(*, name: str) -> Component:
    def Heading() -> Component:
        <h1>Hello, {name}</h1>

    <{Heading} />
```

Add `@subcomponent` when callers also need the nested component:

```hyper
from hyper import Component, subcomponent

def Page(*, title: str) -> Component:
    @subcomponent
    def Header(*, title: str) -> Component:
        <header>{title}</header>

    <{Header} title={title} />
```

`Header` receives its own props and does not capture parent render state:

```python
page = Page(title="Home")
page.render()

other_header = Page.Header(title="Other")
other_header.render()
```

```html
<header>Home</header>
<header>Other</header>
```

`Page.Header` is read-only. Without `@subcomponent`, `Header` remains local and `Page.Header` does not exist.

---

## Multiple Components Per File

Every template so far has defined one component named after its file.

Group related components in one library file. Create `components/forms.hyper`:

```hyper
from hyper import Component

def Form(*, action: str) -> Component:
    <form {action}>
        {...}
    </form>
end

def Input(*, name: str, type: str = "text") -> Component:
    <input {name} {type} />
end

def Button(*, type: str = "submit") -> Component:
    <button {type}>
        {...}
    </button>
end
```

Create `pages/Login.hyper`:

```hyper
from app.components.forms import Form, Input, Button

---

<{Form} action="/login">
    <{Input} name="email" type="email" />
    <{Input} name="password" type="password" />
    <{Button}>Sign In</{Button}>
</{Form}>
```

```python
from app.pages import Login
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

## Use Python Helpers

Define imports and helper functions above `---`.

Create `components/Article.hyper`:

```hyper
from datetime import datetime

def format_date(d: datetime) -> str:
    return d.strftime("%B %d, %Y")
end

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
from app.components import Article
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
from hyper import safe

html_content: str

---

<div>{safe(html_content)}</div>
```

Only use `safe()` for content you trust (e.g., sanitized HTML from your database).

---

## Streaming

Call a component to render all its HTML:

```python
from app.pages import Feed

html = Feed(posts=all_posts).render()
```

Pass `stream=True` to send chunks as they render:

```python
from fastapi.responses import StreamingResponse

@app.get("/feed")
def feed():
    return StreamingResponse(
        Feed(posts=all_posts).render(stream=True),
        media_type="text/html",
    )
```

Write the component normally:

```hyper
posts: list[Post]

---

<div class="feed">
    for post in posts:
        <article>{post.title}</article>
    end
</div>
```

---

## File Structure

Each file is one implicit component or one component library.

Use `---` to select an implicit component and separate setup from rendering:

```hyper
from utils import helper

name: str
count: int = 0

---

greeting = helper(name)

<div>
    <h1>{greeting}</h1>
    <span>{count}</span>
</div>
```

Put imports, helpers, constants, and props above `---`. Put rendering code below it.

Plain HTML needs no separator:

```hyper
<div>Hello World</div>
```

A component library needs no separator either:

```hyper
from hyper import Component

def Header(*, title: str) -> Component:
    <header>{title}</header>
end

def Footer() -> Component:
    <footer>Copyright 2024</footer>
end
```

```python
from app.components.layout import Header, Footer
```

Top-level output selects an implicit component even without `---`:

```hyper
<h1>Hello</h1>
```

```python
from app.pages import Home
```

Declarations and Python without rendered output select library mode:

```hyper
from hyper import Component

DEFAULT_TITLE = "Home"

def Header(*, title: str = DEFAULT_TITLE) -> Component:
    <header>{title}</header>
end
```

```python
from app.components.layout import DEFAULT_TITLE, Header
```

Normal Python takes precedence when both files exist:

```text
Home.py
Home.hyper
```

```python
from app.pages import Home  # Home.py
```

Implicit components must live inside a package. Root-level `.hyper` files work only as component libraries.
