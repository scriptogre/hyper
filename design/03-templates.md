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

Make a file: `app/pages/Layout.py`

```python
# app/pages/Layout.py

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

**Note:** Layouts use **PascalCase** naming (e.g., `Layout.py`, `AuthLayout.py`) which signals they're not routes. The router automatically skips PascalCase files when generating routes.

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
# app/pages/index.py
from app.pages import Layout

t'''
<{Layout}>
    <h1>Welcome Home!</h1>
    <p>This is the homepage.</p>
</{Layout}>
'''
```

Content between `<{Layout}>` and `</{Layout}>` replaces `{...}`.

**Import tip:** Since layouts are in `/pages`, you import them just like any other module: `from app.pages import Layout`. No aliasing needed because the file is already named `Layout.py`.

## Layout Props

Make layouts dynamic with props.

```python
# app/pages/Layout.py

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
# app/pages/index.py

from app.pages import Layout

t'''
<{Layout} title="Home">
    <h1>Welcome!</h1>
</{Layout}>
'''
```

**Or import for editor support:**

```python
# app/pages/index.py

from app.pages import Layout
from app.pages.Layout import title

t'''
<{Layout} {title}="Home">
    <h1>Welcome!</h1>
</{Layout}>
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
# app/pages/index.py

from app.pages import Layout
from app.pages.Layout import title, lang  # Import props

t'''
<{Layout} {title}="Home" {lang}="en">
    <h1>Welcome!</h1>
</{Layout}>
'''
```

For brevity, you can import all props using `*`:

```python
from app.pages.Layout import *  # Import all props
```

But beware that it may clutter your namespace.

```python
from app.pages.Layout import title, lang
```

</details>

You can also pass variables:

```python
# app/pages/index.py

from datetime import datetime
from app.pages import Layout
from app.pages.Layout import title

t'''
<{Layout} {title}={"Home | " + datetime.now().strftime("%Y")}>
    <h1>Welcome!</h1>
</{Layout}>
'''
```

## Default Slot Content

Slots can have fallback content. Use tag syntax (`<{...}>...</{...}>`).

```python
# app/pages/Layout.py

t'''
<!doctype html>
<html>
<head>
    <title>My Site</title>
</head>
<body>
    <{...}>
        <p>This is default content if slot is empty.</p>
    </{...}>
</body>
</html>
'''
```

If no content is provided, it shows the default. Otherwise, it's replaced.

## Named Slots

Layouts can have multiple content areas.

```python
# app/pages/BlogLayout.py

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
from app.pages import BlogLayout

t'''
<{BlogLayout}>
    <{...} name="sidebar">
        ...
    </{...}>

    <h1>Dashboard Content</h1>
    <p>This goes in the unnamed slot.</p>
</{BlogLayout}>
'''
```

You can also fill slots using attributes, rather than tag syntax:
```python
from app.pages import Dashboard

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
# app/pages/BlogLayout.py
from app.pages import Layout

blog_title: str

t'''
<{Layout} title={f"Blog - {blog_title}"}>
    <div class="blog-container">
        {...}
    </div>
</{Layout}>
'''
```

---

# Components

Components let you break down your UI into reusable or extractable pieces.

## Types of Components

Hyper has two types of components:

1. **Page components** - Live in `/app/pages`, page-specific, can be stateful
2. **Shared components** - Live in `/app/components`, reusable across the app, should be stateless

**Most of the time, you don't need components at all.** Keep HTML inline in your routes during active development.

---

## Page Components

### ⚠️ When NOT to Extract

**Don't extract components if:**

1. **Elements depend on each other** - If you have `hx-get` with `hx-target` pointing to another part of the page, keep them together:

```python
# ❌ BAD - Separated components lose context
# SearchForm.py has hx-target="#results"
# SearchResults.py defines id="results"
# Now you have to jump between files to understand the connection

# ✅ GOOD - Keep coupled elements together
t'''
<div>
    <form hx-get="/search" hx-target="#results">
        <input name="q" />
    </form>
    <div id="results">
        {[t'<div>{result.title}</div>' for result in results]}
    </div>
</div>
'''
```

2. **You're in active development** - Constantly changing things means constantly switching files. Keep it inline until things stabilize.

3. **The component would be used once** - No point extracting something that's only used in one place.

### When to Extract

**Only extract when the route file becomes genuinely hard to read** - you'll know when this happens.

**Start with everything inline in your route:**

```python
# app/pages/posts/index.py
from app.pages import Layout

posts = get_all_posts()

t'''
<{Layout} title="Blog Posts">
    <h1>Recent Posts</h1>

    {[t'''
        <article class="post-card">
            <img src={post.image} />
            <h2>{post.title}</h2>
            <p>{post.excerpt}</p>
            <a href="/posts/{post.id}">Read more</a>
        </article>
    ''' for post in posts]}
</{Layout}>
'''
```

**This inline approach is often clearer than extracting `<PostCard>`.** Everything is visible in one place.

**Only if the route becomes truly cluttered (200+ lines, multiple concerns), consider extracting:**

### Step 1: Add the component file

Create `PostCard.py` as a sibling to your route:

```
posts/
  index.py       # Route file
  PostCard.py    # Component (PascalCase = not a route)
```

### Step 2: Move the HTML

```python
# app/pages/posts/PostCard.py
from app.models import Post

post: Post  # Receives data from parent

t'''
<article class="post-card">
    <img src={post.image} />
    <h2>{post.title}</h2>
    <p>{post.excerpt}</p>
    <a href="/posts/{post.id}">Read more</a>
</article>
'''
```

### Step 3: Import and use

```python
# app/pages/posts/index.py
from app.pages import Layout
from . import PostCard  # Import from same directory

posts = get_all_posts()

t'''
<{Layout} title="Blog Posts">
    <h1>Recent Posts</h1>
    {[t'<{PostCard} post={post} />' for post in posts]}
</{Layout}>
'''
```

✅ **Now your route is clean and readable!**

---

## Complex Pages with Multiple Components

For routes with several components, use a directory structure:

### Example: User Profile Page

**Before (cluttered):**
```python
# app/pages/users/[id].py - 200 lines!
from app.pages import Layout

id: int
user = get_user(id)

t'''
<{Layout} title={user.name}>
    <!-- 30 lines of avatar/header HTML -->
    <!-- 50 lines of edit form HTML -->
    <!-- 40 lines of activity feed HTML -->
    <!-- 30 lines of settings panel HTML -->
</{Layout}>
'''
```

**After (organized):**

### Step 1: Convert to directory

```
users/
  [id]/
    index.py        # Main route
    Avatar.py       # Component
    EditForm.py     # Component
    Activity.py     # Component
    Settings.py     # Component
```

### Step 2: Extract components

```python
# app/pages/users/[id]/index.py
from app.pages import Layout
from . import Avatar, EditForm, Activity, Settings

id: int
user = get_user(id)

t'''
<{Layout} title={user.name}>
    <{Avatar} user={user} />
    <{EditForm} user={user} />
    <{Activity} user={user} />
    <{Settings} user={user} />
</{Layout}>
'''
```

```python
# app/pages/users/[id]/Avatar.py
from app.models import User

user: User

t'''
<header class="profile-header">
    <img src={user.avatar} alt={user.name} />
    <h1>{user.name}</h1>
    <p>{user.bio}</p>
</header>
'''
```

```python
# app/pages/users/[id]/EditForm.py
from app.models import User

user: User

t'''
<form method="POST" action="/users/{user.id}/edit">
    <input name="name" value={user.name} />
    <textarea name="bio">{user.bio}</textarea>
    <button>Save</button>
</form>
'''
```

✅ **Each file is now focused and manageable!**

---

## Adding Page Components to Simple Routes

You can add components even to single-file routes:

```
posts/
  index.py       # Route: /posts
  PostCard.py    # Component (sibling)
  Filters.py     # Component (sibling)
```

```python
# app/pages/posts/index.py
from app.pages import Layout
from . import PostCard, Filters

posts = get_all_posts()

t'''
<{Layout} title="Blog">
    <{Filters} />
    {[t'<{PostCard} post={post} />' for post in posts]}
</{Layout}>
'''
```

**PascalCase files are ignored by the router** - they won't create routes.

---

## Shared Components

**When you copy-paste the same HTML across 3+ different routes**, then consider creating a shared component in `/app/components`.

### Example: Button Component

**You notice you're using this pattern everywhere:**

```python
# app/pages/posts/index.py
t'<button class="btn btn-primary" hx-post="/posts/create">New Post</button>'

# app/pages/users/index.py
t'<button class="btn btn-primary" hx-post="/users/create">New User</button>'

# app/pages/settings/index.py
t'<button class="btn btn-primary" hx-post="/settings/save">Save</button>'
```

### Step 1: Create shared component

```python
# app/components/Button.py
text: str
action: str = ""
variant: str = "primary"

t'''
<button class="btn btn-{variant}" hx-post={action}>
    {text}
</button>
'''
```

### Step 2: Use across routes

```python
# app/pages/posts/index.py
from app.components import Button

t'<{Button} text="New Post" action="/posts/create" />'

# app/pages/users/index.py
from app.components import Button

t'<{Button} text="New User" action="/users/create" />'
```

✅ **One change updates everywhere!**

---

## Component Guidelines

### Page Components vs Shared Components

| **Page Components** | **Shared Components** |
|---------------------|----------------------|
| Live in `/app/pages` | Live in `/app/components` |
| Page-specific, can be stateful | Reusable, should be stateless |
| Extract when cluttered (rarely) | Create after 3+ uses |
| Import: `from . import Form` | Import: `from app.components import Button` |
| PascalCase (e.g., `EditForm.py`) | PascalCase (e.g., `Button.py`) |

### Progression

1. **Start inline** - Write HTML directly in route (stay here as long as possible!)
2. **Extract to page component** - Only when route becomes unmanageably cluttered
3. **Promote to shared component** - Only after copy-pasting across 3+ routes

**Premature extraction makes projects harder to work with:**
- Switching between files during development is tedious
- Coupled elements (like `hx-target` pairs) become obscured
- Changes require navigating multiple files
- Inline code is often clearer, especially with simple loops

Keep it simple. Extract only when the pain of not extracting exceeds the pain of extracting.

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

## Partials

Partials let you render only part of a page for HTMX updates.

### Mark a Partial

Add `{partial}` to any element with an `id`:

```python
from hyper import partial

users = User.all()

t'''
<div>
    <h1>Users</h1>
    <ul {partial} id="user-list">
        {[t'<li>{user.name}</li>' for user in users]}
    </ul>
</div>
'''
```

### Return a Partial

Use `render()` to return only specific partials:

```python
from hyper import partial, render

users = User.all()

t'''
<div>
    <h1>Users</h1>
    <ul {partial} id="user-list">
        {[t'<li>{user.name}</li>' for user in users]}
    </ul>
</div>
'''

# Return only the user-list partial
render(partial="user-list")
```

The template runs completely. Only the `user-list` partial is rendered in the response.

### HTMX Example

Combine partials with page components for clean HTMX patterns:

```python
# app/pages/users/index.py
from hyper import partial, render, Header
from app.pages import Layout
from . import UserList

hx_request: bool | None = Header("HX-Request")

users = get_all_users()

t'''
<{Layout} title="Users">
    <h1>All Users</h1>
    <{UserList} users={users} {partial}="user-list" />
</{Layout}>
'''

if hx_request:
    render(partial="user-list")
```

```python
# app/pages/users/UserList.py
from app.models import User

users: list[User]

t'''
<div class="user-grid">
    {[t'<div class="user-card">{user.name}</div>' for user in users]}
</div>
'''
```

**First request:** Full page with layout
**HTMX requests:** Just the `UserList` component

### Multiple Partials

Specify which partials to return:

```python
from hyper import partial, render

t'''
<div {partial} id="stats">
    Users: {User.count()}
</div>

<div {partial} id="activity">
    Last login: {last_login}
</div>
'''

render(partials=["stats", "activity"])
```

### Named Partials

Set partial names explicitly instead of using `id`:

```python
from hyper import partial

t'''
<div {partial}="user-list">
    ...
</div>
'''
```

### Dynamic Partial Names

Generate partial names programmatically:

```python
from hyper import partial

users = User.all()

t'''
<div id="user-list">
    {[t'<div {partial}={f"user-{user.id}"}>{user.name}</div>' for user in users]}
</div>
'''

render(partial=f"user-{user_id}")
```

Useful for updating individual items in a list.

### Multiple Top-Level Tags as a Partial

Use `<{Fragment}>` to group multiple elements into one partial, without adding extra wrapper tags:

```python
from hyper import Fragment, partial

t'''
<{Fragment} {partial}="multi-tags">
    <div>
        ...
    </div>
    <span>
        ...
    </span>
</{Fragment}>
'''
```

The entire `<{Fragment}>...</{Fragment}>` block is treated as a single partial named `multi-tags`.

---

## Whitespace Control

Hyper removes unwanted whitespace from templates automatically.

### The Default Behavior

Templates handle whitespace like Jinja2.

**Without whitespace control:**
```python
users = ["Alice", "Bob"]

t'''
<ul>
    {[t'<li>{user}</li>' for user in users]}
</ul>
'''
```

Would render as:
```html
<ul>

    <li>Alice</li><li>Bob</li>
</ul>
```

Extra blank line after `<ul>`. Indentation issues.

**With Hyper's defaults (already enabled):**
```html
<ul>
    <li>Alice</li><li>Bob</li>
</ul>
```

Clean output. No extra whitespace.

### How It Works

Two settings handle whitespace (both enabled by default):

**`trim_newlines`** - Removes newline after `{expressions}` and `{[...]}`. Same as Jinja2's `trim_blocks`.

**`trim_indent`** - Removes leading whitespace before `{expressions}` and `{[...]}`. Same as Jinja2's `lstrip_blocks`.

### Whitespace Is Preserved Where It Matters

Inside these elements, whitespace is kept:
- `<pre>` - Preformatted text
- `<code>` - Code blocks
- `<textarea>` - Form inputs
- `<script>` - JavaScript
- `<style>` - CSS

```python
t'''
<pre>
    {code}
</pre>
'''
```

Indentation inside `<pre>` stays intact.

### The `:empty` Selector Problem

CSS `:empty` selector requires truly empty elements. Template whitespace breaks this:

```python
message: str = ""

t'''
<div class="empty:hidden">
    {message}
</div>
'''
```

Renders as:
```html
<div class="empty:hidden">

</div>
```

The div contains whitespace. `:empty` doesn't match.

**Solution: Use `{trim}="all"`**

```python
from hyper import trim

message: str = ""

t'''
<div class="empty:hidden" {trim}="all">
    {message}
</div>
'''
```

Renders as:
```html
<div class="empty:hidden"></div>
```

Truly empty. The `:empty` selector works.

**Note:** `{trim}="all"` only removes whitespace between tags. Text content stays normal:
```python
t'''<div {trim}="all">Hello world</div>'''
```
Renders: `<div>Hello world</div>` (the space in "Hello world" is preserved)

<details>
<summary><strong>Advanced:</strong> Global Settings</summary>

Configure defaults in `main.py`:

```python
from hyper import Hyper

app = Hyper(
    templates={
        "trim_newlines": True,   # Default
        "trim_indent": True,     # Default
    }
)
```

Most projects never need to change these.

</details>

<details>
<summary><strong>Advanced:</strong> Per-Element Control</summary>

Override trimming for specific elements:

```python
from hyper import trim

# Custom trim settings
t'''<div {trim}={{"newlines": False, "indent": True}}>{content}</div>'''

# Force trimming inside <pre> (override automatic exception)
t'''<pre {trim}={{"newlines": True, "indent": True}}>{code}</pre>'''
```

Rarely needed in practice.

</details>

---

## Migrating from Jinja2

### Template Syntax

| Jinja2                                 | Hyper                                                          |
|----------------------------------------|----------------------------------------------------------------|
| `{% extends "base.html" %}`            | `from app.pages import Layout` then `<{Layout}>...</{Layout}>` |
| `{% block content %}...{% endblock %}` | `{...}` or `<{...} name="content"/>`                           |
| `{% include "header.html" %}`          | `from app.pages import Header` then `<{Header}/>`              |
| `{% if condition %}`                   | `{condition and t'<div>...</div>'}`                            |
| `{% for item in items %}`              | `{[t'<div>{item}</div>' for item in items]}`                   |
| `{{ variable }}`                       | `{variable}`                                                   |
| `{{ variable\|safe }}`                 | `{variable:safe}`                                              |
| `{# comment #}`                        | `<!--# comment #-->`                                           |
| `{% set x = value %}`                  | `x = value` (at module top)                                    |

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