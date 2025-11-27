# Content Collections

Organize structured data in `app/content/`. Query it from your pages.

---

## Your First Collection

Create a content directory:

```
app/content/
  blog/
    intro.md
    python-tips.md
```

Add a type hint:

```python
# app/content/__init__.py

blogs: list[dict]
```

The CLI finds the markdown files. The `blogs` list is populated automatically.

**That's it.** No schema. No configuration. Just works.

---

## Simple vs Type-Safe

Start simple. Add structure when you need it.

### Simple (Dictionaries)

```python
# app/content/__init__.py

blogs: list[dict]  # CLI infers path from name: app/content/blog/
```

Use in pages:

```python
from app.content import blogs

recent = sorted(blogs, key=lambda b: b["date"], reverse=True)

t"""
<h1>{post["title"]}</h1>
"""
```

**When to use:**
- Quick prototypes
- Simple blogs
- You trust your content

### Type-Safe (Dataclasses)

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

Use in pages:

```python
from app.content import blogs

recent = sorted(blogs, key=lambda b: b.date, reverse=True)  # IDE autocomplete!

t"""
<h1>{post.title}</h1>
"""
```

**When to use:**
- Need validation
- Want IDE autocomplete
- Team projects
- Production sites

The CLI validates content against your schema. Catches errors at build time.

---

## Use in Pages

Import and use the collection:

```python
# app/pages/blog/index.py
from app.content import blogs

# Filter and sort with Python
published = [b for b in blogs if b.published]
recent = sorted(published, key=lambda b: b.date, reverse=True)

t"""
<html>
<body>
    <h1>Blog Posts</h1>
    {[t'''
        <article>
            <h2>{post.title}</h2>
            <p>By {post.author} on {post.date}</p>
        </article>
    ''' for post in recent]}
</body>
</html>
"""
```

Full IDE autocomplete. Type-safe. Pure Python.

---

## Markdown Files

Create markdown files with frontmatter:

```markdown
---
title: "Introduction to Hyper"
slug: "intro"
date: 2025-01-15
author: "Chris"
tags: ["python", "web"]
published: true
---

# Introduction to Hyper

This is the content of the post.

## Features

- File-based routing
- Static generation
```

The frontmatter (YAML between `---`) maps to dataclass fields.

Add `content` and `html` fields to access the markdown:

```python
@dataclass
class Blog:
    title: str
    slug: str
    date: date
    content: str  # Raw markdown
    html: str     # Rendered HTML

    class Meta:
        path = "app/content/blog"
```

Use in templates:

```python
post = next(b for b in blogs if b.slug == slug)

t"""
<article>
    <h1>{post.title}</h1>
    <div>{post.html}</div>
</article>
"""
```

---

## Generate Pages from Content

Use content to generate static pages:

```python
# app/pages/blog/[slug].py
from typing import Literal
from app.content import blogs

# Path parameter (injected by framework)
slug: Literal[*[b.slug for b in blogs]]

# ---

post = next(b for b in blogs if b.slug == slug)

t"""
<html>
<body>
    <h1>{post.title}</h1>
    <p class="meta">By {post.author} on {post.date}</p>
    <div>{post.html}</div>
</body>
</html>
"""
```

The framework generates one page per blog post.

---

## Dataclass Methods

Add methods to make content easier to work with:

```python
from dataclasses import dataclass
from datetime import date

@dataclass
class Blog:
    title: str
    slug: str
    date: date
    author: str
    tags: list[str] = []

    class Meta:
        path = "app/content/blog"

    @property
    def url(self) -> str:
        """Get the post's URL"""
        return f"/blog/{self.slug}"

    @property
    def formatted_date(self) -> str:
        """Format date for display"""
        return self.date.strftime("%B %d, %Y")

    def has_tag(self, tag: str) -> bool:
        """Check if post has a specific tag"""
        return tag in self.tags
```

Use in templates:

```python
t"""
<a href="{post.url}">{post.title}</a>
<time>{post.formatted_date}</time>
"""
```

---

## Multiple Collections

Define multiple collections:

```python
# app/content/__init__.py
from dataclasses import dataclass

@dataclass
class Blog:
    title: str
    slug: str

    class Meta:
        path = "app/content/blog"

@dataclass
class Author:
    name: str
    bio: str
    avatar: str

    class Meta:
        path = "app/content/authors"

blogs: list[Blog]
authors: list[Author]
```

Directory structure:

```
app/content/
  blog/
    intro.md
  authors/
    chris.yaml
```

Import both:

```python
from app.content import blogs, authors

author = next(a for a in authors if a.name == "Chris")
```

---

## File Formats

### Markdown (.md)

```markdown
---
title: "Post Title"
slug: "post-title"
---

# Content here
```

Frontmatter + markdown content.

### JSON (.json)

```json
{
  "name": "Acme Corp",
  "logo": "/logos/acme.png",
  "tier": "gold"
}
```

### YAML (.yaml, .yml)

```yaml
name: Chris
bio: Framework creator
avatar: /images/chris.jpg
social:
  twitter: "@chris"
  github: "chris"
```

### TOML (.toml)

```toml
name = "Acme Corp"
logo = "/logos/acme.png"
tier = "gold"
```

All formats work. The CLI parses them automatically.

---

## Single File Content

Load a single configuration file:

```python
# app/content/__init__.py
from dataclasses import dataclass

@dataclass
class SiteConfig:
    title: str
    description: str
    twitter: str
    github: str

    class Meta:
        path = "app/content/site.toml"

config: SiteConfig  # Single instance, not a list
```

File:

```toml
# app/content/site.toml
title = "My Site"
description = "A great site"
twitter = "@mysite"
github = "mysite"
```

Use in pages:

```python
from app.content import config

t"""
<html>
<head>
    <title>{config.title}</title>
    <meta name="description" content={config.description}>
</head>
<body>
    ...
</body>
</html>
"""
```

---

## Simple Lists

For simple data without a schema:

```python
# app/content/__init__.py

sponsors: list[dict]  # No dataclass needed

class _SponsorsConfig:
    class Meta:
        path = "app/content/sponsors.json"
```

File:

```json
[
  {"name": "Acme", "logo": "/logos/acme.png"},
  {"name": "TechCo", "logo": "/logos/techco.png"}
]
```

Use:

```python
from app.content import sponsors

t"""
{[t'<img src={s["logo"]} alt={s["name"]}>' for s in sponsors]}
"""
```

---

## Nested Data

Dataclasses support nested structures:

```python
from dataclasses import dataclass

@dataclass
class Social:
    twitter: str
    github: str
    website: str | None = None

@dataclass
class Author:
    name: str
    bio: str
    social: Social

    class Meta:
        path = "app/content/authors"

authors: list[Author]
```

YAML file:

```yaml
# app/content/authors/chris.yaml
name: Chris
bio: Framework creator
social:
  twitter: "@chris"
  github: "chris"
  website: "https://example.com"
```

Use:

```python
author = authors[0]
print(author.social.twitter)  # "@chris"
```

---

## Optional Fields

Use `| None` and defaults for optional fields:

```python
from dataclasses import dataclass

@dataclass
class Blog:
    title: str
    slug: str

    # Optional fields
    excerpt: str | None = None
    tags: list[str] = []
    published: bool = True
    featured: bool = False

    class Meta:
        path = "app/content/blog"
```

Files without these fields work fine. Defaults are used.

---

## Validation

The CLI validates content at build time:

```python
@dataclass
class Blog:
    title: str        # Required
    date: date        # Must be valid date
    tags: list[str]   # Must be a list
```

**Build errors are clear:**

```
blog/intro.md validation failed:
  'title' is required
  'date' must be valid (got "2025-13-99")
  'tags' must be a list (got string)
```

Fix the file. Build again. Errors disappear.

---

## Filtering and Sorting

Use Python list comprehensions:

```python
from app.content import blogs
from datetime import date

# Filter by field
published = [b for b in blogs if b.published]

# Filter by multiple conditions
recent = [b for b in blogs
          if b.published and b.date > date(2025, 1, 1)]

# Filter by tag
python_posts = [b for b in blogs if "python" in b.tags]

# Sort by date (newest first)
sorted_posts = sorted(blogs, key=lambda b: b.date, reverse=True)

# Get first 10
top_posts = sorted(blogs, key=lambda b: b.date, reverse=True)[:10]
```

Pure Python. No framework methods needed.

---

## Entry IDs

Each entry gets an `id` field automatically:

```python
@dataclass
class Blog:
    id: str  # Filename without extension
    title: str
    slug: str
```

For `app/content/blog/intro.md`, the `id` is `"intro"`.

For nested files:
- `app/content/blog/tutorials/intro.md` → `id = "tutorials/intro"`

Use IDs for lookups:

```python
post = next(b for b in blogs if b.id == "intro")
```

---

## Subdirectories

Organize content with subdirectories:

```
app/content/
  blog/
    tutorials/
      intro.md
      advanced.md
    reviews/
      product-a.md
```

Filter by path:

```python
from app.content import blogs

# Get all tutorials
tutorials = [b for b in blogs if b.id.startswith("tutorials/")]

# Get all reviews
reviews = [b for b in blogs if b.id.startswith("reviews/")]
```

The `id` includes the full path relative to the collection.

---

## References Between Collections

Reference other content:

```python
@dataclass
class Blog:
    title: str
    author_id: str  # References an Author

    class Meta:
        path = "app/content/blog"

@dataclass
class Author:
    name: str
    bio: str

    class Meta:
        path = "app/content/authors"

blogs: list[Blog]
authors: list[Author]
```

Resolve references manually:

```python
from app.content import blogs, authors

post = blogs[0]
author = next(a for a in authors if a.id == post.author_id)

print(f"{post.title} by {author.name}")
```

Or add a helper method:

```python
@dataclass
class Blog:
    title: str
    author_id: str

    def get_author(self) -> Author:
        from app.content import authors
        return next(a for a in authors if a.id == self.author_id)
```

Use:

```python
author = post.get_author()
```

---

## Custom Parsing

Add custom logic when parsing files:

```python
from dataclasses import dataclass, field
from datetime import date

@dataclass
class Blog:
    title: str
    date: date
    reading_time: int = field(init=False)

    class Meta:
        path = "app/content/blog"

    def __post_init__(self):
        """Calculate reading time from content"""
        if hasattr(self, 'content'):
            words = len(self.content.split())
            self.reading_time = max(1, words // 200)
```

The CLI calls `__post_init__` after creating each instance.

---

## Zero Dependencies

Content collections use only Python stdlib:
- `dataclasses` (Python 3.7+)
- `datetime`, `pathlib`, `json`, etc.

No framework dependencies. No runtime imports. Pure Python.

The CLI handles parsing. Your code just imports data.

---

## Build Time Loading

Content is loaded once at build time:

```bash
hyper build
```

**What happens:**
1. CLI scans `app/content/__init__.py`
2. Finds `blogs: list[Blog]` annotation
3. Reads `Blog.Meta.path` → `"app/content/blog"`
4. Parses all files in that directory
5. Validates against `Blog` dataclass
6. Injects `blogs` list when executing pages
7. Generates static HTML

No runtime overhead. Everything is pre-loaded.

---

## Best Practices

### Naming Conventions

- **Files:** lowercase-with-dashes.md
- **Collections:** plural (blogs, authors, sponsors)
- **Classes:** singular PascalCase (Blog, Author, Sponsor)

### Schema Design

```python
@dataclass
class Blog:
    # Required fields first
    title: str
    slug: str
    date: date

    # Optional fields with defaults
    excerpt: str | None = None
    tags: list[str] = []
    published: bool = True
```

Keep required fields minimal. Use optional for everything else.

### Methods vs Properties

```python
@dataclass
class Blog:
    date: date

    @property
    def formatted_date(self) -> str:
        """Use property for computed values"""
        return self.date.strftime("%B %d, %Y")

    def has_tag(self, tag: str) -> bool:
        """Use method when taking arguments"""
        return tag in self.tags
```

---

## Example: Complete Blog

### Directory Structure

```
app/content/
  blog/
    intro.md
    python-tips.md
  authors/
    chris.yaml
```

### Content Definition

```python
# app/content/__init__.py
from dataclasses import dataclass
from datetime import date

@dataclass
class Blog:
    id: str
    title: str
    slug: str
    date: date
    author_id: str
    content: str
    html: str
    tags: list[str] = []
    published: bool = True

    class Meta:
        path = "app/content/blog"

    @property
    def url(self) -> str:
        return f"/blog/{self.slug}"

    @property
    def formatted_date(self) -> str:
        return self.date.strftime("%B %d, %Y")

@dataclass
class Author:
    id: str
    name: str
    bio: str
    avatar: str

    class Meta:
        path = "app/content/authors"

blogs: list[Blog]
authors: list[Author]
```

### Markdown File

```markdown
---
title: "Introduction to Hyper"
slug: "intro"
date: 2025-01-15
author_id: "chris"
tags: ["python", "web"]
published: true
---

# Introduction to Hyper

This is the content.
```

### List Page

```python
# app/pages/blog/index.py
from app.content import blogs

published = [b for b in blogs if b.published]
sorted_posts = sorted(published, key=lambda b: b.date, reverse=True)

t"""
<html>
<body>
    <h1>Blog Posts</h1>
    {[t'''
        <article>
            <h2><a href="{post.url}">{post.title}</a></h2>
            <time>{post.formatted_date}</time>
            <p>{", ".join(post.tags)}</p>
        </article>
    ''' for post in sorted_posts]}
</body>
</html>
"""
```

### Detail Page

```python
# app/pages/blog/[slug].py
from typing import Literal
from app.content import blogs, authors

# Path parameter (injected by framework)
slug: Literal[*[b.slug for b in blogs]]

# ---

post = next(b for b in blogs if b.slug == slug)
author = next(a for a in authors if a.id == post.author_id)

t"""
<html>
<body>
    <article>
        <h1>{post.title}</h1>
        <p class="meta">
            By {author.name} on {post.formatted_date}
        </p>
        <div>{post.html}</div>
    </article>
</body>
</html>
"""
```

---

## Python at Build Time

Use any Python library during builds.

### Analyze Content

```python
# app/pages/stats.py
from app.content import blogs
import pandas as pd

df = pd.DataFrame([{'date': b.date, 'words': len(b.content.split())} for b in blogs])

avg_words = df['words'].mean()
posts_per_month = df.groupby(df['date'].dt.to_period('M')).size()

t"""
<html>
<body>
    <h1>Statistics</h1>
    <p>Average: {avg_words:.0f} words</p>
    <ul>
        {[t'<li>{month}: {count}</li>' for month, count in posts_per_month.items()]}
    </ul>
</body>
</html>
"""
```

The DataFrame code runs once at build time.

### Fetch External Data

```python
# app/pages/books/index.py
from app.content import books
import httpx

for book in books:
    resp = httpx.get(f"https://api.example.com/books/{book.isbn}")
    data = resp.json()
    book.cover = data['cover_url']
    book.rating = data['rating']

t"""
<div class="books">
    {[t'<img src="{b.cover}"> {b.title} ({b.rating}⭐)' for b in books]}
</div>
"""
```

API calls happen at build time. Results are baked into static HTML.

### Install Any Library

```bash
pip install pandas httpx pillow matplotlib
```

Import and use in your pages. Everything runs during `hyper build`.

---

## Type Inference and IDE Integration

### Auto-Generate Types from Frontmatter

Hyper scans content files at build time and infers dataclass definitions from frontmatter keys and values. Developers get full IDE autocomplete without writing schemas.

**How it works:**
1. Scan `app/content/**/*.md` for YAML frontmatter
2. Infer Python types from values (strings, dates, arrays, objects)
3. Generate `app/_hyper_generated/content.pyi` with full type hints
4. User code imports normally; type checkers see rich types

**Example:**

```
# Developer writes only markdown
# app/content/blog/my-post.md
***
title: "Understanding Python"
date: 2025-01-15
tags: [python, tutorial]
***

# Build output (auto-generated)
# app/_hyper_generated/content.pyi

@dataclass
class Blog:
    title: str
    date: date
    tags: list[str]

blogs: list[Blog]
```

**IDE Integration:**
```
from app.content import blogs

for post in blogs:
    print(post.date)    # ✅ IDE knows type is `date`
    print(post.invalid) # ❌ Caught by type checker
```

### Type Safety Constraints

Enforce consistency to prevent silent bugs:

- **Strict validation:** All files must have consistent field types. Missing optional fields must be explicit.
- **Inference fallback:** If fields conflict (e.g., string in one file, int in another), fail at build with clear error message.
- **Optional override:** Allow `schema_overrides` to specify custom types when inference isn't precise enough.

```
from hyper import auto_collection

# Simple: auto-infer everything
blogs = auto_collection("app/content/blog/*.md")

# Advanced: override complex types
from dataclasses import dataclass

@dataclass
class Author:
    name: str
    email: str

blogs = auto_collection(
    "app/content/blog/*.md",
    schema_overrides={"author": Author}
)
```

### Generated Files and `.gitignore`

Generated type stubs live in `app/_hyper_generated/` and must be added to `.gitignore`. Hyper maintains them automatically during `hyper dev` and `hyper build`.

Developers should never edit generated files directly; they're regenerated on each build.

```
# .gitignore
app/_hyper_generated/
```

---

## Key Points

- **Start simple with `list[dict]`**
- **Add dataclasses for structure**
- **Use glob patterns in `Meta.path`**
- **Import any library at build time**
- **CLI loads content once during build**
- **Zero runtime dependencies**

---

**[← Previous: Static Site Generation](08-ssg.md)** | **[Back to Index](README.md)** | **[Next: Advanced Features →](10-advanced.md)**
