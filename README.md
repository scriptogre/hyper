# Hyper

**A Python-powered static site generator built for speed**

Write static sites with Python 3.14's t-strings. Full Python ecosystem at build time. Zero runtime dependencies.

```python
# app/pages/index.py

t"""
<html>
<head>
    <title>Welcome to Hyper</title>
</head>
<body>
    <h1>Hello, Hyper!</h1>
    <p>Built with Python. Served as static HTML.</p>
</body>
</html>
"""
```

---

## Key Features

### 🗂️ File-Based Routing

Your file structure **is** your URL structure.

```
app/pages/
  index.py              → /
  about.py              → /about
  blog/
    index.py            → /blog
    [slug].py           → /blog/:slug
```

### 🎨 Native T-String Templates

Python 3.14's t-strings provide type-safe templates with zero learning curve.

```python
# app/pages/team.py

members = get_team_members()

t"""
<html>
<body>
    <h1>Our Team</h1>
    {[t'<div>{member.name} - {member.role}</div>' for member in members]}
</body>
</html>
"""
```

### 📝 Content Collections

Organize content with Python dataclasses or plain dictionaries.

```python
# app/content/__init__.py
from dataclasses import dataclass
from datetime import date

@dataclass
class Blog:
    title: str
    slug: str
    date: date

    class Meta:
        path = "app/content/blog/*.md"

blogs: list[Blog]
```

```python
# app/pages/blog/index.py
from app.content import blogs

recent = sorted(blogs, key=lambda b: b.date, reverse=True)[:10]

t"""
<h1>Recent Posts</h1>
{[t'<a href="/blog/{post.slug}">{post.title}</a>' for post in recent]}
"""
```

### 🎯 Dynamic Routes

Generate pages from content automatically.

```python
# app/pages/blog/[slug].py
from typing import Literal
from app.content import blogs

# Path parameter (injected by framework)
slug: Literal[*[b.slug for b in blogs]]

# ---

post = next(b for b in blogs if b.slug == slug)

t"""
<article>
    <h1>{post.title}</h1>
    <div>{post.html}</div>
</article>
"""
```

### 🚀 The Power of Python

Use any Python library at build time.

```python
# app/pages/stats.py
from app.content import blogs
import pandas as pd

df = pd.DataFrame([{'date': b.date, 'words': len(b.content.split())} for b in blogs])
avg_words = df['words'].mean()

t"""
<h1>Blog Statistics</h1>
<p>Average: {avg_words:.0f} words per post</p>
"""
```

Need data analysis? Image processing? API calls? Just `pip install` and use it.

### 📄 Markdown Files

Write content in Markdown with frontmatter and Python execution blocks.

```markdown
---
title: "My Post"
date: 2025-01-15
---

# {frontmatter.title}

```python exec
recent_posts = get_recent_posts(5)
```

## Related Posts
{[t'<li>{p.title}</li>' for p in recent_posts]}
```

### ⚡ Built for Speed

- Rust CLI for millisecond builds
- Incremental builds
- Instant hot reload
- Single binary distribution

---

## Installation

```bash
# Via uv (recommended):
uvx hyper init myblog
cd myblog
uvx hyper dev

# Or install:
uv tool install hyper
hyper build
```

---

## Documentation

### Getting Started

- **[Architecture](design/00-architecture.md)** - Technical decisions and implementation plan
- **[Overview](design/01-overview.md)** - What is Hyper and why use it

### SSG Documentation

1. **[Routing](design/02-routing.md)** - File-based routing, dynamic routes
2. **[Templates](design/03-templates.md)** - t-strings, layouts, components
3. **[Markdown](design/04-markdown.md)** - Markdown files with Python
4. **[Static Site Generation](design/08-ssg.md)** - Building static HTML, path generation
5. **[Content Collections](design/09-content.md)** - Organize and query content

See [design/00-index.md](design/00-index.md) for complete documentation index.

---

## What Makes Hyper Different?

**Compared to Hugo/Zola/Jekyll:**
- Full Python at build time (not template language)
- Any PyPI package available
- Python dataclasses for content
- Type-safe with IDE autocomplete

**Compared to Astro:**
- Simpler API - no `getStaticPaths()` boilerplate
- Python instead of JavaScript
- More flexible content organization
- Rust CLI for speed

**Compared to Python frameworks:**
- Zero runtime dependencies
- Pure static output
- No server needed
- Deploy anywhere

---

## Philosophy

### Simple Things Simple

```python
# app/content/__init__.py
blogs: list[dict]  # That's it. No schema needed.
```

### Powerful When Needed

```python
@dataclass
class Blog:
    title: str
    date: date

    class Meta:
        path = "app/content/blog/*.md"

    @property
    def url(self) -> str:
        return f"/blog/{self.slug}"

blogs: list[Blog]  # Full validation, IDE autocomplete, type safety
```

---

## Status

**Current:** Planning & design phase
**Next:** Rust CLI implementation
**Future:** SSR mode (server-side rendering)

See [design/00-architecture.md](design/00-architecture.md) for implementation roadmap.

---

## Links

- **Documentation:** [design/00-index.md](design/00-index.md)
- **GitHub:** https://github.com/yourusername/hyper
- **Python 3.14 t-strings:** [PEP 750](https://peps.python.org/pep-0750/)

---

## License

MIT
