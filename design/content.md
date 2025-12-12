# Content

Organize structured data. Write content in markdown.

***

# Content Collections

Store data in `app/content/`. Query from pages.

## Simple Start

```
app/content/
  blog/
    intro.md
    tips.md
```

Add type hint:

```python
# app/content/__init__.py

blogs: list[dict]
```

Done. CLI finds markdown files. Populates `blogs` list automatically.

## Type-Safe Collections

Add structure when needed:

```python
# app/content/__init__.py
from dataclasses import dataclass
from datetime import date

@dataclass
class Blog:
    title: str
    slug: str
    date: date
    content: str  # Raw markdown
    html: str     # Rendered HTML

    class Meta:
        path = "app/content/blog/*.md"

blogs: list[Blog]
```

Use in pages:

```python
# app/pages/blog/index.py
from app.content import blogs

recent = sorted(blogs, key=lambda b: b.date, reverse=True)[:10]

t"""
<h1>Recent Posts</h1>
{[t'<a href="/blog/{post.slug}">{post.title}</a>' for post in recent]}
"""
```

## Markdown Files

Create files with frontmatter:

```markdown
---
title: "Introduction to Hyper"
slug: "intro"
date: 2025-01-15
tags: ["python", "web"]
---

# Introduction

Content here.
```

Frontmatter maps to dataclass fields.

## Generate Pages from Content

```python
# app/pages/blog/[slug].py
from typing import Literal
from app.content import blogs

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

Framework generates one page per blog post.

## Dataclass Methods

Add helpers:

```python
@dataclass
class Blog:
    title: str
    slug: str
    date: date

    class Meta:
        path = "app/content/blog"

    @property
    def url(self) -> str:
        return f"/blog/{self.slug}"

    @property
    def formatted_date(self) -> str:
        return self.date.strftime("%B %d, %Y")
```

Use in templates:

```python
t'<a href="{post.url}">{post.title}</a>'
t'<time>{post.formatted_date}</time>'
```

## File Formats

### Markdown (.md)

```markdown
---
title: "Post Title"
---

Content here.
```

### JSON (.json)

```json
{
  "name": "Acme Corp",
  "tier": "gold"
}
```

### YAML (.yaml)

```yaml
name: Chris
bio: Creator
```

### TOML (.toml)

```toml
name = "Acme Corp"
tier = "gold"
```

CLI parses all formats automatically.

## Single File Config

```python
# app/content/__init__.py
@dataclass
class SiteConfig:
    title: str
    description: str

    class Meta:
        path = "app/content/site.toml"

config: SiteConfig  # Single instance
```

```toml
# app/content/site.toml
title = "My Site"
description = "A great site"
```

Use:

```python
from app.content import config

t'<title>{config.title}</title>'
```

## Multiple Collections

```python
# app/content/__init__.py
@dataclass
class Blog:
    title: str

    class Meta:
        path = "app/content/blog"

@dataclass
class Author:
    name: str

    class Meta:
        path = "app/content/authors"

blogs: list[Blog]
authors: list[Author]
```

## Filtering and Sorting

Pure Python:

```python
from app.content import blogs

# Filter
published = [b for b in blogs if b.published]
python_posts = [b for b in blogs if "python" in b.tags]

# Sort
recent = sorted(blogs, key=lambda b: b.date, reverse=True)

# Limit
top_10 = recent[:10]
```

## Optional Fields

```python
@dataclass
class Blog:
    title: str
    slug: str

    # Optional
    excerpt: str | None = None
    tags: list[str] = []
    published: bool = True
```

## Entry IDs

Each entry gets automatic `id` field:

```python
@dataclass
class Blog:
    id: str  # Filename without extension
    title: str
```

For `app/content/blog/intro.md`, `id = "intro"`.

For `app/content/blog/tutorials/intro.md`, `id = "tutorials/intro"`.

***

# Markdown Pages

Place `.md` files in `app/pages/`. They become pages.

```
app/pages/
  about.md              → /about
  blog/
    post-1.md           → /blog/post-1
```

## Basic Markdown

```markdown
# About Us

We build **great software**.

- Fast
- Reliable
```

Renders to HTML.

## Frontmatter

Add YAML between `---` delimiters:

```markdown
---
title: "About Us"
date: 2025-01-15
---

# {frontmatter.title}

Published on {frontmatter.date}.
```

Access with `{frontmatter.field}`.

## Python Execution Blocks

Run Python at build time. Use ` ```python exec ` blocks:

```markdown
---
title: "Blog"
---

```python exec
from app.content import blogs
recent = sorted(blogs, key=lambda b: b.date, reverse=True)[:5]
```

## Recent Posts

{[t'- [{p.title}]({p.url})' for p in recent]}
```

Variables become available in markdown.

## Layouts for Markdown

Wrap markdown in layout. Create `Layout.py` in same folder:

```python
# app/pages/blog/Layout.py

html: str          # Rendered markdown
frontmatter: dict  # YAML frontmatter
content: str       # Raw markdown

t"""
<html>
<head>
    <title>{frontmatter.get('title', 'Blog')}</title>
</head>
<body>
    <article>
        {html}
    </article>
</body>
</html>
"""
```

Layout receives `html`, `frontmatter`, and `content`.

## Dynamic Routes with Markdown

```markdown
<!-- app/pages/blog/[slug].md -->
---
---

```python exec
from typing import Literal
from app.content import blogs

slug: Literal[*[b.slug for b in blogs]]

post = next(b for b in blogs if b.slug == slug)
```

# {post.title}

Published on {post.date}

{post.content}
```

## Multiple Execution Blocks

Variables persist across blocks:

```markdown
```python exec
from app.content import blogs
total = len(blogs)
```

We have {total} posts.

```python exec
avg_words = sum(len(b.content.split()) for b in blogs) // total
```

Average: {avg_words} words.
```

## When to Use Markdown

**Use markdown for:**
- Blog posts
- Documentation
- Content-heavy pages

**Use Python for:**
- Complex logic
- Multiple layouts
- Component composition

***

# Ordering Content

Files sort alphabetically. Control with prefixes.

## Number Prefixes

```
app/content/
  chapters/
    01-intro.md
    02-basics.md
    03-advanced.md
```

## Access Order Field

```python
@dataclass
class Chapter:
    id: str
    order: int  # Auto-extracted from prefix
    title: str
```

Sort by order:

```python
sorted_chapters = sorted(chapters, key=lambda c: c.order)
```

For `01-intro.md`, `order = 1`.
For `002-setup.md`, `order = 2`.
No prefix means `order = 0`.

***

# Python at Build Time

Use any library during builds.

```python
# app/pages/stats.py
from app.content import blogs
import pandas as pd

df = pd.DataFrame([{'date': b.date, 'words': len(b.content.split())} for b in blogs])
avg_words = df['words'].mean()

t"""
<h1>Statistics</h1>
<p>Average: {avg_words:.0f} words per post</p>
"""
```

Install libraries:

```bash
pip install pandas httpx pillow matplotlib
```

Use in pages. Runs during `hyper build`.

---

**[← Previous: Templates](templates.md)** | **[Next: Static Site Generation →](ssg.md)**
