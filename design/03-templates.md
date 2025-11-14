# Templates & Layouts

HyperProse uses Python 3.14's t-string templates powered by tdom for composable, type-safe HTML.

---

## Layouts & Template Composition

Reuse common HTML structure across pages using tdom's component system.

### Creating a Layout

1. Create a layout file in `layouts/` with a PascalCase filename
2. Use `Slot` type for HTML content placeholders
3. Use regular Python types for simple props

```python
# layouts/Base.py
from hyperprose import Slot

# Simple props (passed as attributes)
title: str = "My App"

# HTML content slots (passed via markers)
content: Slot

t"""
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <nav>
        <a href="/">Home</a>
        <a href="/about">About</a>
        <a href="/contact">Contact</a>
    </nav>

    <main>
        {content}
    </main>

    <footer>
        <p>&copy; 2025 My App</p>
    </footer>
</body>
</html>
"""
```

**Key points:**
- Layout files live in `layouts/` directory
- Use **PascalCase** filenames (e.g., `Base.py`, `Blog.py`)
- Just a bare t-string at module level - no variable assignment needed
- `Slot` type = HTML content placeholder
- Regular types (`str`, `int`, `bool`) = simple props

### Using a Layout

Import the layout module and use slot markers for HTML content:

```python
# routes/index.py
from layouts import Base

title = "Homepage"

t"""
<{Base} title={title}>
    <h1>Welcome Home!</h1>
    <p>This is the homepage content.</p>
    <ul>
        <li>Built with HyperProse</li>
        <li>Powered by Python 3.14</li>
        <li>Using tdom for templates</li>
    </ul>
</{Base}>
"""
```

**How it works:**
- `<{Base}>` invokes the component
- `title="..."` passes simple prop as an attribute
- Inner content replaces `{content}` in the layout

### Multiple Slots

Layouts can have multiple named slots:

```python
# layouts/Dashboard.py
from hyperprose import Slot

title: str = "Dashboard"
content: Slot
sidebar: Slot

t"""
<!doctype html>
<html>
<head>
    <title>{title}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <aside class="sidebar">
        {sidebar}
    </aside>

    <main>
        {content}
    </main>
</body>
</html>
"""
```

```python
# routes/dashboard.py
from layouts import Dashboard

t"""
<{Dashboard}>
    {Dashboard.sidebar:}
      <nav>
          <a href="/settings">Settings</a>
          <a href="/profile">Profile</a>
      </nav>
    {Dashboard.sidebar:end}

    {Dashboard.content:}
      <h1>Dashboard</h1>
      <div class="stats">
          <p>Your stats here...</p>
      </div>
    {Dashboard.content:end}
</{Dashboard}>
"""
```

You can also import & use the slot variables directly:
```python
from layouts import Dashboard
from layouts.Dashboard import content, sidebar

t"""
<{Dashboard}>
    {sidebar:}
      <nav>
          <a href="/settings">Settings</a>
          <a href="/profile">Profile</a>
      </nav>
    {sidebar:end}

    {content:}
      <h1>Dashboard</h1>
      <div class="stats">
          <p>Your stats here...</p>
      </div>
    {content:end}
</{Dashboard}>
"""
```

You can also skip the `:end` marker if you want.

```python
from layouts import Dashboard
from layouts.Dashboard import content, sidebar
t"""
<{Dashboard}>
    {sidebar:}
        <nav>
            <a href="/settings">Settings</a>
            <a href="/profile">Profile</a>
        </nav>

    {content:}
        <h1>Dashboard</h1>
        <div class="stats">
            <p>Your stats here...</p>
        </div>
</{Dashboard}>
"""
```

You can also skip the slot marker for the main `content`, but then the other slots need to use `:end` markers:

```python
from layouts import Dashboard
from layouts.Dashboard import sidebar

t"""
<{Dashboard}>
    {sidebar:}
        ...
    {sidebar:end}
    
    <!-- Everything else goes to the content slot -->
    <h1>Dashboard</h1>
    <div class="stats">
        <p>Your stats here...</p>
    </div>
</{Dashboard}>
"""
```

**Key points:**
- Use `{Component.slotname:}` or `{slotname:}` markers to target specific slots
- Markers partition the content - everything between markers goes to that slot
- Use `{Component.slotname:end}` or `{slotname:end}` to close a slot section (optional)
- If only one slot named `content` exists, markers are optional
- Content outside any slot markers goes to the `content` slot by default (in which case other slots need explicit `:end` markers)

### Nested Layouts

Layouts can wrap other layouts:

```python
# layouts/Base.py
from hyperprose import Slot

content: Slot

t"""
<!doctype html>
<html>
<body>{content}</body>
</html>
"""
```

```python
# layouts/Blog.py
from hyperprose import Slot
from layouts import Layout

content: Slot

t"""
<{Layout} title="Blog">
    <div class="blog-container">
        <aside class="sidebar">
            <h3>Categories</h3>
            <ul>
                <li><a href="/blog/tech">Tech</a></li>
                <li><a href="/blog/design">Design</a></li>
            </ul>
        </aside>
        <article class="blog-content">
            {content}
        </article>
    </div>
</{Layout}>
"""
```

```python
# routes/blog/{slug}.py
from layouts import Blog

slug: str
post = Post.get(slug=slug)

t"""
<{Blog}>
    <h1>{post.title}</h1>
    <div class="post-meta">
        <span>By {post.author}</span>
        <span>{post.date}</span>
    </div>
    <div class="post-content">
        {post.content:safe}
    </div>
</{Blog}>
"""
```

---

## Components

Components are reusable template modules stored in `components/`. Use PascalCase filenames for the component name.

### Creating Components

**Simple component (props only):**

```python
# components/UserCard.py
from models import User

user: User               # Required
show_email: bool = True  # Optional

t"""
<div class="user-card">
    <div class="user-avatar">
        <img src="{user.avatar_url}" alt="{user.name}">
    </div>
    <h3>{user.name}</h3>
    {show_email and t'<p class="email">{user.email}</p>'}
    <a href="/users/{user.id}">View Profile</a>
</div>
"""
```

```python
# components/Button.py

text: str              # Required
type: str = "primary"  # Optional
disabled: bool = False

t"""
<button class="btn btn-{type}" disabled={disabled}>
    {text}
</button>
"""
```

**Component with slots:**

```python
# components/Alert.py
from hyperprose import Slot

message: str  # Simple prop
type: str = "info"
content: Slot  # HTML content slot

t"""
<div class="alert alert-{type}" role="alert">
    <strong>{message}</strong>
    <div class="alert-content">
        {content}
    </div>
</div>
"""
```

### Using Components

```python
# routes/users/index.py
from models import User
from layouts import Layout
from components import UserCard, Alert, Button

users: list[User] = User.all()

t"""
<{Layout} page_title="Users">
    <{Alert} message="User Directory" type="success">
        <p>Browse all registered users below.</p>
    </{Alert}>

    <div class="users-grid">
        {[t'<{UserCard} user={user} show_email={True} />' for user in users]}
    </div>

    <{Button} text="Add New User" type="primary" />
</{Layout}>
"""
```

**Notes:**
- Self-closing syntax `<{Component} />` for components without slots

---

## Safe HTML Rendering

By default, all variables are escaped for security. To render trusted HTML:

### Using `Markup`

```python
# routes/blog/{slug}.py
from hyperprose import Markup
from models import Post

slug: str
post: Post = Post.get(slug=slug)

# Mark trusted HTML as safe (won't be escaped)
safe_content = Markup(post.html_content)

t"""
<article>
    <h1>{post.title}</h1>
    <div class="content">
        {safe_content}
    </div>
</article>
"""
```

### Using `:safe` Format Specifier

```python
t"""
<article>
    <h1>{post.title}</h1>
    <div class="content">
        {post.html_content:safe}
    </div>
</article>
"""
```

---

## Advanced Attribute Handling

HyperProse templates are powered by tdom, which provides sophisticated attribute handling for common HTML patterns.

### The `class` Attribute

The `class` attribute has special handling to make it easy to combine classes from different sources.

**Basic list syntax:**

```python
classes = ["btn", "btn-primary", "active"]
button = t'<button class={classes}>Click me</button>'
# <button class="btn btn-primary active">Click me</button>
```

**Conditional classes with dictionaries:**

```python
is_active = True
is_disabled = False

classes = ["btn", "btn-primary", {"active": is_active, "disabled": is_disabled}]
button = t'<button class={classes}>Click me</button>'
# <button class="btn btn-primary active">Click me</button>
```

**Mixed syntax - combine strings, lists, dicts, and conditionals:**

```python
classes = [
    "btn",
    "btn-primary",
    {"active": user.is_active},
    user.is_admin and "admin-controls",
    None,  # Ignored
]
button = t'<button class={classes}>Click me</button>'
```

**Real-world example:**

```python
# routes/posts/{post_id}.py
from layouts import Base

post_id: int
post = Post.get(id=post_id)
user = get_current_user()

is_author = user.id == post.author_id
can_edit = is_author or user.is_admin

t"""
<{Base} title={post.title}>
    <article class={["post", {"featured": post.is_featured, "draft": post.is_draft}]}>
        <h1>{post.title}</h1>
        <div class={["actions", {"visible": can_edit}]}>
            <button class={["btn", {"btn-primary": can_edit}]}>Edit</button>
        </div>
    </article>
</{Base}>
"""
```

### The `style` Attribute

In addition to strings, you can provide a dictionary of CSS properties for the `style` attribute:

```python
styles = {
    "color": "red",
    "font-weight": "bold",
    "margin": "10px",
    "background-color": "#f0f0f0"
}
element = t'<p style={styles}>Important text</p>'
# <p style="color: red; font-weight: bold; margin: 10px; background-color: #f0f0f0">Important text</p>
```

**Dynamic styles:**

```python
# routes/dashboard.py
from layouts import Base

theme = get_user_theme()
is_dark_mode = theme.mode == "dark"

background_color = "#1a1a1a" if is_dark_mode else "#ffffff"
text_color = "#ffffff" if is_dark_mode else "#000000"

t"""
<{Base}>
    <div style={{"background-color": {background_color}, "color": {text_color}, "padding": "20px"}}>
        <h1>Welcome to your dashboard</h1>
    </div>
</{Base}>
"""
```

### The `data` and `aria` Attributes

The `data` and `aria` attributes have special handling to convert dictionary keys to proper attribute names:

```python
data_attrs = {"user-id": 123, "role": "admin", "profile-complete": True}
aria_attrs = {"label": "Close dialog", "hidden": True, "live": "polite"}

element = t'<div data={data_attrs} aria={aria_attrs}>Content</div>'
# <div data-user-id="123" data-role="admin" data-profile-complete="true"
#      aria-label="Close dialog" aria-hidden="true" aria-live="polite">Content</div>
```

**Note:** Boolean values in `aria` attributes are converted to `"true"` or `"false"` strings as per the ARIA specification.

**Real-world example with HTMX:**

```python
# components/LoadMoreButton.py

page: int = 1
total_pages: int = 10

t"""
<button
    class={["btn", "btn-secondary", {"disabled": page >= total_pages}]}
    hx-get="/api/posts"
    hx-vals={{"page": page + 1}}
    hx-target="#posts-container"
    hx-swap="beforeend"
    data={{"current-page": page, "total-pages": total_pages}}
    aria={{"label": f"Load more posts (page {page + 1} of {total_pages})"}}>
    Load More
</button>
"""
```

### Attribute Spreading

You can spread multiple attributes at once using a dictionary with curly braces:

```python
attrs = {
    "href": "https://example.com",
    "target": "_blank",
    "rel": "noopener noreferrer"
}
link = t'<a {attrs}>External link</a>'
# <a href="https://example.com" target="_blank" rel="noopener noreferrer">External link</a>
```

**Combining spreading with individual attributes:**

```python
base_attrs = {"id": "my-button", "type": "button"}
disabled = True

button = t'<button {base_attrs} disabled={disabled} class="btn">Click me</button>'
# <button id="my-button" type="button" disabled class="btn">Click me</button>
```

**Special attributes work with spreading:**

```python
classes = ["btn", {"active": True}]
attrs = {
    "class": classes,
    "id": "submit-btn",
    "data": {"action": "submit", "form-id": 123}
}

button = t'<button {attrs}>Submit</button>'
# <button class="btn active" id="submit-btn" data-action="submit" data-form-id="123">Submit</button>
```

**Real-world component example:**

```python
# components/Link.py
from typing import Any

href: str
text: str
external: bool = False
attrs: dict[str, Any] = {}

# Automatically add external link attributes
if external:
    attrs["target"] = "_blank"
    attrs["rel"] = "noopener noreferrer"

t'<a href={href} {attrs}>{text}</a>'
```

```python
# routes/index.py
from components import Link

t"""
<nav>
    <{Link} href="/" text="Home" />
    <{Link} href="/about" text="About" />
    <{Link} href="https://example.com" text="External" external={True} />
</nav>
"""
# Results in:
# <nav>
#     <a href="/">Home</a>
#     <a href="/about">About</a>
#     <a href="https://example.com" target="_blank" rel="noopener noreferrer">External</a>
# </nav>
```

### Boolean Attributes

Boolean attributes are handled automatically:

```python
is_disabled = True
is_readonly = False

form_input = t'<input type="text" disabled={is_disabled} readonly={is_readonly} />'
# <input type="text" disabled>
# Note: readonly is omitted because it's False
```

**Common boolean attributes:**
- `disabled`
- `readonly`
- `required`
- `checked`
- `selected`
- `multiple`
- `autofocus`

### Multiple Substitutions in Attributes

You can use multiple variable substitutions within a single attribute:

```python
first = "Alice"
last = "Smith"
user_id = 123

button = t'<button data-name="{first} {last}" data-user-id={user_id}>Profile</button>'
# <button data-name="Alice Smith" data-user-id="123">Profile</button>
```

---

## Conditional Rendering

HyperProse provides the `_if` helper for conditional rendering of template sections. This allows you to show or hide entire blocks of HTML based on runtime conditions.

### Block Conditional Rendering

Use `{_if(condition):}` to start a conditional block and `{_if:end}` to close it:

```python
from hyperprose import Slot, _if

title: str = "Dashboard"
content: Slot
sidebar: Slot = None  # Optional slot

t"""
<!doctype html>
<html>
<head>
    <title>{title}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    {_if(sidebar):}
        <aside class="sidebar">
            {sidebar}
        </aside>
    {_if:end}

    <main>
        {content}
    </main>
</body>
</html>
"""
```

**Key points:**
- Opening syntax mirrors Python: `{_if(condition):}`
- Closing marker is explicit: `{_if:end}`
- Uses `_if` (with underscore) since `if` is a Python reserved keyword
- Works with any Python boolean expression

### Conditional Patterns

**Check if slot/variable exists:**

```python
from hyperprose import Slot, _if

content: Slot
header: Slot = None
footer: Slot = None

t"""
<div class="page">
    {_if(header):}
        <header>{header}</header>
    {_if:end}

    <main>{content}</main>

    {_if(footer):}
        <footer>{footer}</footer>
    {_if:end}
</div>
"""
```

**Boolean flags:**

```python
from hyperprose import _if

show_nav: bool = True
is_admin: bool = False

t"""
<html>
<body>
    {_if(show_nav):}
        <nav>
            <a href="/">Home</a>
            <a href="/about">About</a>
            {_if(is_admin):}
                <a href="/admin">Admin Panel</a>
            {_if:end}
        </nav>
    {_if:end}

    <main>Content here</main>
</body>
</html>
"""
```

**Comparison operators:**

```python
from hyperprose import _if

user_count: int = 42
items: list = get_items()

t"""
<html>
<body>
    {_if(user_count > 0):}
        <div class="stats">
            We have {user_count} registered users!
        </div>
    {_if:end}

    {_if(len(items) > 10):}
        <div class="warning">
            Showing first 10 of {len(items)} items
        </div>
    {_if:end}

    <div class="items">
        {[t'<div>{item}</div>' for item in items[:10]]}
    </div>
</body>
</html>
"""
```

**Complex conditions:**

```python
from hyperprose import _if

user = get_current_user()
is_author: bool = user.id == post.author_id
can_edit: bool = is_author or user.is_admin

t"""
<article>
    <h1>{post.title}</h1>
    <div>{post.content}</div>

    {_if(can_edit):}
        <div class="actions">
            <a href="/posts/{post.id}/edit">Edit</a>
            <button hx-delete="/posts/{post.id}">Delete</button>
        </div>
    {_if:end}
</article>
"""
```

### Conditional Element Rendering

For even cleaner syntax, you can use `{_if(condition)}` directly on an element to conditionally render it. No `:` or `:end` markers needed!

```python
from hyperprose import _if

user = get_current_user()
is_admin = user.is_admin

t"""
<nav>
    <a href="/">Home</a>
    <a href="/about">About</a>
    <a href="/admin" {_if(is_admin)}>Admin Panel</a>
</nav>
"""
# If is_admin is True:  <nav><a href="/">Home</a><a href="/about">About</a><a href="/admin">Admin Panel</a></nav>
# If is_admin is False: <nav><a href="/">Home</a><a href="/about">About</a></nav>
```

**This is perfect for:**
- Conditionally showing navigation links
- Optional form fields
- Dynamic buttons based on permissions
- Any single element that should appear/disappear

**Real-world examples:**

```python
# routes/profile.py
from hyperprose import _if
from layouts import Base

user = get_current_user()
can_edit = user.id == profile_user.id or user.is_admin
is_verified = profile_user.is_verified

t"""
<{Base} title={profile_user.name}>
    <div class="profile">
        <h1>{profile_user.name}</h1>
        <span class="badge" {_if(is_verified)}>✓ Verified</span>

        <div class="profile-content">
            <p>{profile_user.bio}</p>
        </div>

        <button class="btn-edit" {_if(can_edit)}>Edit Profile</button>
        <button class="btn-admin" {_if(user.is_admin)}>Admin Actions</button>
    </div>
</{Base}>
"""
```

**Comparison:**

```python
# ❌ Verbose block syntax for a single element
{_if(is_admin):}
    <a href="/admin">Admin</a>
{_if:end}

# ✅ Clean shorthand for single elements
<a href="/admin" {_if(is_admin)}>Admin</a>
```

**When to use which:**
- Use `{_if(condition)}` shorthand → For single elements
- Use `{_if(condition):}` ... `{_if:end}` → For blocks with multiple elements

### Why `_if` Instead of Python `if`?

You might wonder why not just use Python's native `if` statement. Here's why `_if` is better:

**❌ Python `if` duplicates condition checks:**

```python
# Not recommended - condition checked twice
show_sidebar = True

t"""
<body>
    {show_sidebar and t'''
        <aside class="sidebar">
            <nav>...</nav>
        </aside>
    '''}
    <main>Content</main>
</body>
"""
```

**✅ `_if` is cleaner and more maintainable:**

```python
# Recommended - clear intent, no duplication
show_sidebar: bool = True

t"""
<body>
    {_if(show_sidebar):}
        <aside class="sidebar">
            <nav>...</nav>
        </aside>
    {_if:end}
    <main>Content</main>
</body>
"""
```

**Benefits of `_if`:**
- ✅ Condition evaluated once
- ✅ Clear opening and closing markers
- ✅ Easy to scan and understand template logic
- ✅ Better editor support for matching blocks
- ✅ Works seamlessly with tdom's rendering engine

---

## Key Points

- **PascalCase filenames** in `layouts/` and `components/` directories
- **Bare t-strings** at module level - no variable assignment needed
- **Import modules directly**: `from layouts import Base`
- **Two prop types**:
  - Regular types (`str`, `int`, `bool`) → pass as attributes: `<{Base} title="...">`
  - `Slot` type → pass via markers: `{Base.content:}<div>...</div>`
- **Slot marker syntax**: `{Component.slotname:}` partitions content
- **Advanced attributes powered by tdom**:
  - `class` accepts lists and dicts: `class={["btn", {"active": True}]}`
  - `style` accepts dicts: `style={{"color": "red", "margin": "10px"}}`
  - `data` and `aria` expand dicts: `data={{"user-id": 123}}`
  - Attribute spreading: `<a {attrs}>` spreads multiple attributes at once
- **Conditional rendering**:
  - Block syntax: `{_if(condition):}` ... `{_if:end}` for multiple elements
  - Element syntax: `<a href="/admin" {_if(is_admin)}>` for single elements
- **Editor-friendly**: `Component.slotname` resolves correctly in IDEs
- **Components can nest** other components
- **All output escaped** by default for security
- **Use `Markup()` or `:safe`** for trusted HTML

---

**[← Previous: Routing](02-routing.md)** | **[Back to Index](README.md)** | **[Next: Fragments →](04-fragments.md)**
