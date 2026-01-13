# Content

> **Status**: Draft

Load structured data from files.

***

# Loading Content

Use `load()` to read files.

## Without Type Checking

```python
from hyper import load

# Single file
settings = load("settings.json")  # dict

# Multiple files
posts = load("posts/*.json")  # list[dict]
```

No validation. Returns raw dicts.

## With Type Checking

Use `Collection` or `Singleton` base classes:

```python
from hyper import Collection

class Post(Collection):
    title: str
    date: str

    class Meta:
        pattern = "posts/*.json"

# Load all posts
posts = Post.load()  # list[Post]
```

Auto-validates against your schema.

***

# Markdown Content

Use `MarkdownCollection` or `MarkdownSingleton` for markdown files.

## Simple Example

```python
from hyper import MarkdownCollection

class Post(MarkdownCollection):
    title: str
    date: str

    class Meta:
        pattern = "posts/*.md"

posts = Post.load()
```

Markdown files look like:

```markdown
---
title: "My First Post"
date: "2025-01-15"
---

# Hello World

This is the **content**.
```

## Magic Markdown Fields

`MarkdownCollection` provides automatic fields:

```python
post = posts[0]

post.id        # "my-first-post" (filename without extension)
post.slug      # "my-first-post" (URL-friendly)
post.body      # "# Hello World\n\nThis is the **content**."
post.html      # "<h1>Hello World</h1><p>This is the <strong>content</strong>.</p>"
post.title     # "My First Post" (from frontmatter)
post.date      # "2025-01-15" (from frontmatter)
```

**Built-in fields:**
- `id` - Filename without extension
- `slug` - URL-friendly identifier
- `body` - Raw markdown text
- `html` - Rendered HTML

**Custom fields** come from frontmatter.

## Table of Contents

Extract headings and generate TOC:

```python
from hyper import MarkdownCollection

class Post(MarkdownCollection):
    title: str

    class Meta:
        pattern = "posts/*.md"

post = posts[0]

# Access headings
for heading in post.headings:
    print(f"{'  ' * (heading.level - 1)}{heading.text}")
    print(f"  → #{heading.slug}")

# Nested TOC structure
toc = post.toc.nested()  # Returns hierarchical structure
```

***

# Validation Libraries

Choose your preferred validation library.

## With Dataclass (Auto)

`Collection` and `Singleton` auto-apply `@dataclass`:

```python
from hyper import Collection

class Post(Collection):
    title: str
    date: str

    class Meta:
        pattern = "posts/*.json"
```

No explicit decorator needed.

## With Pydantic

```python
import pydantic
from hyper import Collection

class Post(Collection, pydantic.BaseModel):
    title: str
    date: str

    class Meta:
        pattern = "posts/*.json"
```

Get Pydantic validation and features.

## With msgspec

```python
import msgspec
from hyper import Collection

class Post(Collection, msgspec.Struct):
    title: str
    date: str

    class Meta:
        pattern = "posts/*.json"
```

Fast serialization with msgspec.

***

# Collections vs Singletons

## Collection (Multiple Files)

```python
from hyper import Collection

class Post(Collection):
    title: str

    class Meta:
        pattern = "posts/*.md"

posts = Post.load()  # list[Post]
```

Returns a list.

## Singleton (Single File)

```python
from hyper import Singleton

class Config(Singleton):
    site_name: str
    theme: str

    class Meta:
        pattern = "config.json"

config = Config.load()  # Config (single instance)
```

Returns a single object.

***

# File Formats

All formats auto-detected by extension.

## JSON

```json
{
  "title": "My Post",
  "tags": ["python", "web"]
}
```

## YAML

```yaml
title: My Post
tags:
  - python
  - web
```

## TOML

```toml
title = "My Post"
tags = ["python", "web"]
```

## Markdown

```markdown
---
title: "My Post"
tags: ["python", "web"]
---

Content here.
```

***

# Patterns

Use glob patterns to find files.

```python
class Post(Collection):
    title: str

    class Meta:
        # Match specific directory
        pattern = "posts/*.md"

        # Recursive search
        pattern = "posts/**/*.md"

        # Multiple extensions
        pattern = "posts/*.{md,json}"

        # Multiple directories
        pattern = "{posts,articles}/*.md"
```

**Glob syntax:**
- `*` - Match any characters
- `**` - Match any depth (recursive)
- `{a,b}` - Match either `a` or `b`

***

# Using in Pages

Import and use content in pages.

```python
# app/content/__init__.py
from hyper import MarkdownCollection

class Post(MarkdownCollection):
    title: str
    date: str

    class Meta:
        pattern = "content/posts/*.md"

posts = Post.load()
```

Use in pages:

```python
# app/pages/blog.py
from app.content import posts

# Filter
recent = [p for p in posts if p.date > "2025-01-01"]

# Sort
sorted_posts = sorted(posts, key=lambda p: p.date, reverse=True)

# Render
t"""
<h1>Blog</h1>
{[t'<article><h2>{post.title}</h2></article>' for post in sorted_posts[:10]]}
"""
```

Pure Python. No special query language.

***

# Computed Properties

Add methods to your models:

```python
from hyper import MarkdownCollection
from datetime import datetime

class Post(MarkdownCollection):
    title: str
    date: str

    class Meta:
        pattern = "posts/*.md"

    @property
    def url(self) -> str:
        return f"/blog/{self.slug}"

    @property
    def formatted_date(self) -> str:
        dt = datetime.fromisoformat(self.date)
        return dt.strftime("%B %d, %Y")

    @property
    def word_count(self) -> int:
        return len(self.body.split())
```

Use in templates:

```python
t"""
<a href="{post.url}">{post.title}</a>
<time>{post.formatted_date}</time>
<span>{post.word_count} words</span>
"""
```

***

# Optional Fields

```python
from hyper import Collection

class Post(Collection):
    title: str
    slug: str

    # Optional
    excerpt: str | None = None
    tags: list[str] = []
    published: bool = True

    class Meta:
        pattern = "posts/*.md"
```

***

# Entry IDs

Each entry gets automatic `id` field.

For `content/posts/intro.md`:
- `id = "content/posts/intro"`

For nested files, includes full path.

Override in frontmatter if needed:

```markdown
---
id: "my-custom-id"
title: "Post Title"
---
```

***

# Direct load() Usage

Use `load()` directly without classes:

```python
from hyper import load

# Load without validation
posts = load("posts/*.json")  # list[dict]
config = load("config.json")  # dict

# With type hints
posts = load[list[dict]]("posts/*.json")
config = load[dict]("config.json")
```

Good for quick scripts. Use classes for validation.

---

# Pagination (Planned)

> **Status**: 🔮 Exploring design

Split large collections into pages for SSG.

## Concept

```python
from hyper import Page, paginate

posts = Post.load()

# Create pages of 10 items each
pages = paginate(posts, page_size=10)
```

Returns a `Page` type with navigation:

```python
@dataclass
class Page(Generic[T]):
    data: list[T]       # Items on this page
    current_page: int   # 1, 2, 3...
    last_page: int      # Total pages
    total: int          # Total items
    url: PageURL        # .previous, .next, .first, .last
```

Use with `generate()` for SSG:

```hyper
from content import posts
from hyper import paginate

def generate() -> list[Template]:
    for page in paginate(posts, page_size=10):
        yield Template(page=page)

page: Page[Post]

---

<h1>Page {page.current_page} of {page.last_page}</h1>

for post in page.data:
    <article>{post.title}</article>
end

<nav>
    if page.url.previous:
        <a href="{page.url.previous}">← Previous</a>
    end
    if page.url.next:
        <a href="{page.url.next}">Next →</a>
    end
</nav>
```

**Design TBD.**

---

**[← Previous: Templates](templates.md)** | **[Next: Static Site Generation →](ssg.md)**
