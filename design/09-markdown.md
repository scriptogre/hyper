# Markdown Support

Hyper has **first-class markdown support** with full templating power, perfect for blogs, documentation, and content-heavy sites.

---

## Basic Markdown Routes

Create `.md` files in your routes directory:

```markdown
<!-- routes/blog/intro.md -->
---
title: "Introduction to Hyper"
date: 2025-01-15
author: "Chris"
excerpt: "Learn the basics"
layout: "_blog_layout"
prerender: true
---

# Introduction to Hyper

Hyper makes building web apps **fun** and **productive**!

## Features

- File-based routing
- Built-in markdown
- SSG + SSR hybrid
- Full Python power
```

**URL:** `/blog/intro`

**Frontmatter (YAML between `---`):**
- Metadata for the page
- `layout` specifies which layout to use
- `prerender` enables static generation
- All frontmatter values available in templates

---

## Markdown with Layouts

```python
# routes/_blog_layout.py
from hyper import Children, MarkdownMeta

children: Children
meta: MarkdownMeta  # Frontmatter data

BlogLayout = t"""
<!doctype html>
<html>
<head>
    <title>{meta.title} - My Blog</title>
    <meta name="author" content="{meta.author}">
    <meta name="description" content="{meta.excerpt}">
</head>
<body>
    <article>
        <header>
            <h1>{meta.title}</h1>
            <p class="meta">
                By {meta.author} on {meta.date}
            </p>
        </header>
        <div class="content">
            {children}
        </div>
    </article>
</body>
</html>
"""
```

**Key points:**
- `meta: MarkdownMeta` receives frontmatter data
- `children` receives rendered markdown content
- Access frontmatter with `meta.title`, `meta.author`, etc.

---

## Python Execution in Markdown

Use `python exec` code blocks to execute Python code before rendering:

````markdown
---
title: "Dynamic Content"
---

```python exec
from datetime import datetime

# Execute Python code!
now = datetime.now()
year = now.year
users = get_all_users()

# Async works too!
async def get_data():
    global stats
    stats = await fetch_stats()
```

# Hello from {year}!

Current time: {now.strftime("%Y-%m-%d %H:%M:%S")}

We have **{len(users)}** registered users!

Total visits: {stats.visits:,}
````

**Key points:**
- Code in ` ```python exec ` blocks runs before rendering
- Variables become available in markdown
- Supports both sync and async functions
- Use `global` keyword for variables used in markdown

---

## Variables and Expressions

Markdown files support full template interpolation:

```markdown
---
title: "User List"
---

```python exec
users = get_all_users()
total = len(users)
```

# User Directory

We have {total} users registered!

## Users

{'\n'.join([f"- **{u.name}** ({u.email})" for u in users])}

Average posts per user: {sum(u.post_count for u in users) / total:.1f}
```

---

## Components in Markdown

Use tdom components directly in markdown:

```markdown
---
title: "Components Demo"
layout: "_base"
---

```python exec
from routes._components import UserCard, AlertBox, Button

users = get_featured_users()
```

# Featured Users

<{AlertBox} message="These are our top contributors!" type="success" />

{[f'<{{UserCard}} user={{users[{i}]}} />' for i in range(len(users))]}

<{Button} text="View All Users" type="primary" />
```

---

## Dependency Injection in Markdown

Markdown files support the same dependency injection as Python routes:

```markdown
---
title: "User Profile"
prerender: false
---

```python exec
from hyper import Request

# Inject request
request: Request

# Inject path params
user_id: int

# Inject query params
tab: str = "overview"

# Load data
async def load_user():
    global user, posts
    user = await get_user(user_id)
    posts = await get_user_posts(user_id)
```

# {user.name}'s Profile

**Bio:** {user.bio}
**Joined:** {user.created_at}
**Posts:** {len(posts)}

Current tab: **{tab}**
```

---

## Static Markdown with Dynamic Data

Pre-render markdown with build-time data:

```markdown
---
title: "Team Members"
prerender: true
---

```python exec
def get_static_paths():
    teams = get_all_teams()
    return [{"team_id": t.id} for t in teams]

team_id: int
team = get_team(team_id)
members = get_team_members(team_id)
```

# Team: {team.name}

{team.description}

## Members ({len(members)})

{[f'''
### {m.name}
- **Role:** {m.role}
- **Email:** {m.email}
''' for m in members]}
```

---

## Markdown Collections

Get all markdown files from a directory:

```python
# routes/blog/index.py
prerender = True

from hyper import get_collection

# Get all markdown files
posts = get_collection("routes/blog/*.md")

# Filter and sort
published = [p for p in posts if p.meta.get("published", True)]
sorted_posts = sorted(published, key=lambda p: p.meta.date, reverse=True)

t"""
<html>
<body>
    <h1>Blog Posts</h1>
    {[t'''
        <article>
            <h2><a href="{post.url}">{post.meta.title}</a></h2>
            <p class="meta">By {post.meta.author} on {post.meta.date}</p>
            <p>{post.meta.excerpt}</p>
            <a href="{post.url}">Read more →</a>
        </article>
    ''' for post in sorted_posts]}
</body>
</html>
"""
```

**Each post object has:**
- `post.url` - URL path
- `post.meta` - Frontmatter data
- `post.content` - Raw markdown content
- `post.html` - Rendered HTML

---

## Syntax Highlighting

Code blocks are automatically highlighted:

````markdown
Here's some Python code:

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
```

And some JavaScript:

```javascript
const greet = (name) => {
    console.log(`Hello, ${name}!`);
};
```
````

---

## Configuration

```python
# app.py
from hyper import Hyper

app = Hyper(
    markdown_extensions=[
        "fenced_code",      # Code blocks with syntax
        "tables",           # GitHub-style tables
        "footnotes",        # Footnote support
        "toc",              # Table of contents
        "attr_list",        # Add attributes to elements
        "def_list",         # Definition lists
        "abbr",             # Abbreviations
    ],
    syntax_theme="monokai",     # Code highlighting theme
    enable_math=True,           # LaTeX math support
)
```

---

## Complete Blog Example

```
routes/
  _blog_layout.py       # Blog post layout
  blog/
    index.py            # List all posts (static)
    intro.md            # Blog post (static)
    python-tips.md      # Blog post (static)
    {slug}.md           # Dynamic fallback for new posts
```

See full example in FRAMEWORK_DESIGN.md lines 1510-1665.

---

## Key Points

- **`.md` files in routes/ become pages**
- **Frontmatter (YAML) for metadata**
- **` ```python exec ` blocks execute before rendering**
- **`{variable}` interpolation works in markdown**
- **Use components with `<{Component}>`**
- **Dependency injection works same as `.py` routes**
- **`get_collection()` for listing markdown files**
- **Automatic syntax highlighting**
- **Set `prerender: true` in frontmatter for SSG**

---

**[← Previous: Static Site Generation](08-ssg.md)** | **[Back to Index](README.md)** | **[Next: Advanced Features →](10-advanced.md)**
