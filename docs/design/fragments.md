# Fragments

Fragments let you render parts of a template independently. Only the code for that fragment executes.

---

## Why Fragments

A dashboard has a sidebar, content area, and notification count. Each requires different data.

Without fragments, rendering just the sidebar still executes all database calls:

```python
html = await Dashboard(user=user, posts=posts, notifications=notifications)
```

With fragments, render only what you need:

```python
html = await Dashboard.sidebar(notifications=notifications)
```

---

## Defining Fragments

### Single Element

Use `{'fragment'}` on an element with an id:

```python
t"""
<div id="sidebar" {'fragment'}>
  <!--@ for n in {notifications} -->
    <span>{n.text}</span>
  <!--@ end -->
</div>
"""
```

The compiler sees `{'fragment'}`, uses the `id` as the fragment name, and strips the attribute from output.

### Multiple Elements

Use the directive when a fragment has no wrapper element:

```python
t"""
<table>
  <tbody>
    <!--@ fragment rows -->
      <!--@ for row in {data} -->
        <tr><td>{row.name}</td></tr>
      <!--@ end -->
    <!--@ end -->
  </tbody>
</table>
"""
```

---

## Using Fragments

```python
from app.pages import Dashboard

# Full page
html = await Dashboard(user=u, posts=p, notifications=n)

# Single fragment
html = await Dashboard.sidebar(notifications=n)

# Dynamic name
html = await Dashboard.fragment("sidebar", notifications=n)

# Multiple fragments
html = await Dashboard.fragments("sidebar", "content", notifications=n, posts=p)
```

---

## Dependency Analysis

The compiler tracks which props each fragment uses:

```python
user: User
posts: list[Post]
notifications: list[Notification]

t"""
<div id="sidebar" {'fragment'}>{await get_notifications(user)}</div>
<main id="content" {'fragment'}>{await get_posts(user)}</main>
"""
```

Generated signatures:

```python
async def sidebar(user: User) -> str: ...
async def content(user: User) -> str: ...
```

Each fragment only requires the props it actually uses.

---

## Formatters

When rendering multiple fragments, formatters wrap them for transport.

### Built-in Formatters

**HxPartialFormatter** (HTMX 4.0+):

```html
<hx-partial hx-target="#sidebar" hx-swap="innerHTML">
  <div id="sidebar">...</div>
</hx-partial>
```

**OobFormatter** (classic HTMX):

```html
<div id="sidebar">...</div>
<main id="content" hx-swap-oob="true">...</main>
```

**RawFormatter** (no wrapping):

```html
<div id="sidebar">...</div>
<main id="content">...</main>
```

### Configuration

```python
# App-level default
app = Hyper(fragment_formatter=HxPartialFormatter())

# Per-request override
html = await Page.fragments("sidebar", "content", formatter=OobFormatter())
```

### Custom Formatters

```python
class JsonFormatter(FragmentFormatter):
    def format(self, fragments: list[Fragment]) -> str:
        return json.dumps({f.name: f.html for f in fragments})
```

---

## hyper-diff Integration

Fragments and hyper-diff are complementary:

| Optimization | Avoids | Layer |
|--------------|--------|-------|
| Fragments | Unnecessary code execution | Render time |
| hyper-diff | Unnecessary bytes sent | Response time |

hyper-diff is a formatter that computes minimal DOM mutations:

```python
class HyperDiffFormatter(FragmentFormatter):
    def __init__(self, old_dom: str):
        self.old_dom = old_dom

    def format(self, fragments: list[Fragment]) -> str:
        new_dom = assemble(fragments)
        return hyper_diff.diff(self.old_dom, new_dom)
```

Combine both for maximum efficiency:

```python
async def update_profile(request, user_id: int):
    user = await get_user(user_id)

    # Only re-render affected fragment (minimal compute)
    new_header = await Dashboard.header(user=user)

    # Diff for minimal bytes
    old_header = cache.get(f"header:{user.id}")
    return HyperDiffResponse(old_header, new_header)
```

---

## Streaming

Fragments can stream independently:

```python
async for chunk in Dashboard.sidebar.stream(notifications=n):
    yield chunk
```

---

## Errors

Fragment without id:

```
Fragment attribute requires an id.

  You wrote:
    <div {'fragment'}>...</div>

  Add an id:
    <div id="sidebar" {'fragment'}>...</div>
```
a