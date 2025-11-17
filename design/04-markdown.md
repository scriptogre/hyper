# Markdown Files

Write content in Markdown. Add frontmatter for metadata. Execute Python code inside markdown.

---

## Markdown as Pages

Place `.md` files in `app/pages/`. They become HTML pages.

```
app/pages/
  about.md              → /about
  blog/
    post-1.md           → /blog/post-1
```

The file structure defines the URL structure.

---

## Basic Markdown

Write markdown. It renders to HTML.

```markdown
# About Us

We build **great software**.

- Fast
- Reliable
- Simple
```

Becomes:

```html
<h1>About Us</h1>
<p>We build <strong>great software</strong>.</p>
<ul>
  <li>Fast</li>
  <li>Reliable</li>
  <li>Simple</li>
</ul>
```

---

## Frontmatter

Add YAML frontmatter at the top. Use `---` delimiters.

```markdown
---
title: "About Us"
date: 2025-01-15
author: "Jane Doe"
---

# About Us

We build great software.
```

Access frontmatter data with `{frontmatter.field}`:

```markdown
---
title: "My Post"
date: 2025-01-15
---

# {frontmatter.title}

Published on {frontmatter.date}.
```

---

## Python Execution Blocks

Execute Python code inside markdown. Use ` ```python exec ` blocks.

```markdown
---
title: "Blog"
---

# {frontmatter.title}

```python exec
from app.content import blogs
recent = sorted(blogs, key=lambda b: b.date, reverse=True)[:5]
```

## Recent Posts

{[t'- [{p.title}]({p.url})' for p in recent]}
```

The Python code runs at build time. Variables become available in the markdown.

---

## T-String Interpolation

Use t-strings for dynamic content.

```markdown
---
title: "Stats"
---

```python exec
from app.content import blogs
total = len(blogs)
avg_words = sum(len(b.content.split()) for b in blogs) // total
```

# Blog Statistics

We have written **{total}** posts with an average of **{avg_words}** words.
```

---

## Using Layouts

Wrap markdown content in a layout. Create `Layout.py` in the same folder or parent folder.

**app/pages/blog/Layout.py:**

```python
from typing import Annotated

# Injected by framework
html: str
frontmatter: dict

# ---

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

**app/pages/blog/my-post.md:**

```markdown
---
title: "My Post"
---

# {frontmatter.title}

This is my post content.
```

The layout receives `html` (rendered markdown) and `frontmatter`.

---

## Dynamic Routes with Markdown

Use markdown files with dynamic parameters.

**app/pages/blog/[slug].md:**

```markdown
---
# This frontmatter can use injected variables
---

```python exec
from typing import Literal
from app.content import blogs

# Path parameter (injected by framework)
slug: Literal[*[b.slug for b in blogs]]

# ---

# Use injected parameter
post = next(b for b in blogs if b.slug == slug)
```

# {post.title}

Published on {post.date}

{post.content}
```

The `slug` parameter is injected. The markdown generates once per slug value.

---

## Accessing Rendered Content

Layouts receive both raw and rendered content.

Available variables in layouts:

- `html: str` - Rendered markdown as HTML
- `frontmatter: dict` - The YAML frontmatter
- `content: str` - Raw markdown content (without frontmatter)

**Example layout:**

```python
t"""
<html>
<head>
    <title>{frontmatter['title']}</title>
    <meta name="date" content="{frontmatter['date']}">
</head>
<body>
    <h1>{frontmatter['title']}</h1>
    <time>{frontmatter['date']}</time>

    <article>
        {html}
    </article>
</body>
</html>
"""
```

---

## Python in Frontmatter

Frontmatter values are YAML. Python code runs in execution blocks.

**This doesn't work:**

```markdown
---
title: "Blog"
posts: [b.title for b in blogs]  # ❌ YAML doesn't execute Python
---
```

**Do this instead:**

```markdown
---
title: "Blog"
---

```python exec
from app.content import blogs
posts = [b.title for b in blogs]
```

Found {len(posts)} posts.
```

---

## Content Collections in Markdown

Load and query content collections inside markdown.

```markdown
---
title: "All Posts"
---

```python exec
from app.content import blogs
from datetime import date

# Filter posts
recent = [b for b in blogs if b.date.year == 2025]

# Sort posts
recent.sort(key=lambda b: b.date, reverse=True)
```

# {frontmatter.title}

Found {len(recent)} posts from 2025:

{[t'- **{p.title}** ({p.date})' for p in recent]}
```

---

## Multiple Execution Blocks

Use multiple execution blocks. Variables persist across blocks.

```markdown
---
title: "Analysis"
---

```python exec
from app.content import blogs
total = len(blogs)
```

We have {total} posts.

```python exec
# This block sees variables from previous blocks
avg_words = sum(len(b.content.split()) for b in blogs) // total
```

Average length: {avg_words} words.
```

Each block runs in sequence. Later blocks see earlier variables.

---

## When to Use Markdown

**Use markdown for:**
- Blog posts
- Documentation pages
- Content-heavy pages
- Simple layouts

**Use Python (`.py`) for:**
- Complex logic
- Multiple layouts
- Component composition
- Dynamic data processing

---

## Markdown vs Python Pages

**Markdown** (`app/pages/about.md`):

```markdown
---
title: "About"
---

# About Us

We build software.
```

**Python** (`app/pages/about.py`):

```python
t"""
<html>
<head><title>About</title></head>
<body>
    <h1>About Us</h1>
    <p>We build software.</p>
</body>
</html>
"""
```

Both create the same page. Use markdown for content. Use Python for structure.

---

## Nested Layouts

Layouts can nest. Each layout wraps the previous one.

**app/pages/Layout.py** (root layout):

```python
html: str

t"""
<html>
<body>
    <header>Site Header</header>
    {html}
</body>
</html>
"""
```

**app/pages/blog/Layout.py** (blog layout):

```python
html: str
frontmatter: dict

t"""
<article>
    <h1>{frontmatter['title']}</h1>
    {html}
</article>
"""
```

**app/pages/blog/my-post.md:**

```markdown
---
title: "My Post"
---

Content here.
```

The markdown renders through blog layout, then root layout.

---

## Summary

1. Place `.md` files in `app/pages/` - they become pages
2. Add YAML frontmatter between `---` delimiters
3. Execute Python with ` ```python exec ` blocks
4. Use `{frontmatter.field}` to access metadata
5. Use `{variable}` to interpolate Python variables
6. Create `Layout.py` to wrap markdown with HTML structure
7. Layouts receive `html`, `frontmatter`, and `content`
8. Use dynamic routes with `Literal[*[...]]` in execution blocks

---

**Next:** [Dependency Injection](05-dependency-injection.md) - Path params, query params, headers
