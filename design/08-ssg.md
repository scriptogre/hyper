# Static Site Generation

Build static HTML files at build time.

---

## Static Pages

Every file in `app/pages` becomes a URL. 

Create a page file.

```python
# app/pages/about.py

t"""
<html>
<body>
    <h1>About Us</h1>
    <p>We build great software.</p>
</body>
</html>
"""
```

Build the site.

```bash
hyper build
```

**Output:**
```
app/pages/about.py → dist/about/index.html
```

---

## Loading Data

Hyper provides a unified `load` tool. 

It reads files, parses them, and validates them into Python objects using `msgspec`.

```python
from hyper import load, Markdown

posts = load[list[Markdown]]("content/blog/*.md")

# Access fields
for post in posts:
    print(post.title)  # from frontmatter
    print(post.html)   # rendered HTML
    print(post.content)  # raw markdown


```

## Configuration

Configure Hyper in `pyproject.toml`.

### Default Configuration

All settings are optional. Hyper works out of the box with these defaults:

```toml
# pyproject.toml
[tool.hyper]
# Project structure
pages_dir = "app/pages"
content_dirs = ["app/content", "content", "data"]
static_dir = "static"
output_dir = "dist"

# Development
debug = false
dev_server.port = 3000
dev_server.host = "localhost"
dev_server.open = true

# Build
build.clean = true
build.minify = false
build.parallel = true

# Templates
templates.autoescape = true
```

### Common Customizations

```toml
# pyproject.toml
[tool.hyper]
# Use different directory structure
pages_dir = "src/pages"
content_dirs = ["content"]
static_dir = "public"

# Enable debug mode
debug = true

# Customize dev server
dev_server.port = 8000
dev_server.open = false
```

### Configuration Options

**Project Structure:**
- `pages_dir` - Directory containing page files (default: `"app/pages"`)
- `content_dirs` - Content directories searched in order (default: `["app/content", "content", "data"]`)
- `static_dir` - Static assets directory (default: `"static"`)
- `output_dir` - Build output directory (default: `"dist"`)

**Development:**
- `debug` - Enable verbose logging and stack traces (default: `false`)
- `dev_server.port` - Development server port (default: `3000`)
- `dev_server.host` - Development server host (default: `"localhost"`)
- `dev_server.open` - Open browser on server start (default: `true`)

**Build:**
- `build.clean` - Clean output directory before build (default: `true`)
- `build.minify` - Minify HTML/CSS/JS (default: `false`)
- `build.parallel` - Parallel builds for better performance (default: `true`)

**Templates:**
- `templates.autoescape` - Auto-escape HTML in templates (default: `true`)

---

## Pages with Data

Load data at build time.

Define your content.

```python
# app/content/__init__.py

team_members: list[dict] = [
    {"name": "Alice", "role": "CEO"},
    {"name": "Bob", "role": "CTO"},
]
```

Use it in a page.

```python
# app/pages/team.py
from app.content import team_members

t"""
<html>
<body>
    <h1>Our Team</h1>
    {[t'<div>{member["name"]} - {member["role"]}</div>' for member in team_members]}
</body>
</html>
"""
```

Build the site.

```bash
hyper build
```

**Build generates:**
```
dist/team/index.html
```

<details>
<summary>View generated HTML</summary>

```html
<html>
<body>
    <h1>Our Team</h1>
    <div>Alice - CEO</div><div>Bob - CTO</div>
</body>
</html>
```
</details>

The code runs once during build. The result is static HTML.

---

## Loading Content from Files

Load content using the `load()` function.

### Simple Loading

```python
from hyper import loader, Markdown


def generate():
    # Load markdown files
    articles = load[list[Markdown]]("articles/*.md")

    for article in articles:
        yield {"slug": article.slug, "article": article}
```

### Path Resolution

**Relative paths** are searched in content directories:

```python
articles = load[list[Markdown]]("articles/*.md")
```

**CLI searches `content_dirs` from `pyproject.toml`:**
1. `app/content/articles/*.md`
2. `content/articles/*.md`
3. `data/articles/*.md`

Uses the first match found.

**Explicit paths** from project root:

```python
articles = load[list[Markdown]]("app/content/articles/*.md")
config = load[JSON]("config/settings.json")
```

**Absolute paths:**

```python
external = load[list[Markdown]]("/var/data/articles/*.md")
```

**Customize in `pyproject.toml`:**

```toml
[tool.hyper]
content_dirs = ["content", "data", "src/content"]
```

---

### Content Types

Import content types from `hyper`.

```python
from hyper import Markdown, JSON, YAML, TOML
```

**`Markdown`** - For markdown files with frontmatter:
- `id: str` - Filename without extension
- `title: str` - From frontmatter or first `#` header
- `content: str` - Raw markdown body
- `html: str` - Rendered HTML

**`JSON`, `YAML`, `TOML`** - For structured data:
- `id: str` - Filename without extension
- Plus any fields from the file (via Pydantic `extra='allow'`)

---

### Loading Lists vs Single Files

Use `list[Type]` for multiple files, `Type` for single file.

```python
from hyper import loader, Markdown, JSON


def generate():
    # Load multiple files → list
    articles = load[list[Markdown]]("articles/*.md")
    sponsors = load[list[JSON]]("sponsors/*.json")

    # Load single file → single object
    config = load[JSON]("config.json")
    about = load[Markdown]("about.md")
```

**Type-safe:** IDE knows the return type.

---

### Markdown Fields

Markdown files are loaded with special fields.

**File example:**
```markdown
---
title: "My Article"
slug: "my-article"
---

# Article Content

This is the body of the article.
```

**Loaded as `Markdown`:**

```python
from hyper import loader, Markdown


def generate():
    articles = load[list[Markdown]]("articles/*.md")
    article = articles[0]

    # Built-in fields
    article.id  # "my-article" (filename)
    article.title  # "My Article" (from frontmatter)
    article.content  # Raw markdown: "# Article Content\n\n..."
    article.html  # Rendered HTML: "<h1>Article Content</h1>..."

    # Custom fields (from frontmatter)
    article.slug  # "my-article" (works via Pydantic extra='allow')
```

**Use in pages:**

```python
# app/pages/blog/[slug].py
from hyper import loader, Markdown


def generate():
    articles = load[list[Markdown]]("articles/*.md")
    for article in articles:
        yield {"slug": article.slug, "article": article}


slug: str
article: Markdown

# ---

t"""
<h1>{article.title}</h1>
<div>{article.html}</div>
"""
```

---

### Explicit Paths

Specify exact paths with `Annotated`.

```python
from typing import Annotated

sponsors: Annotated[list[dict], "app/content/sponsors.json"]
```

**No auto-discovery.** Uses exactly that file.

**Use when:**
- File is outside `app/content/`
- Name doesn't match variable name
- Single specific file

---

### Multiple Paths

Load from multiple locations. Merge into one list.

```python
from typing import Annotated

articles: Annotated[
    list[dict],
    "app/content/blog/*.md",
    "data/external/*.json"
]
```

**Result:** Files from both paths merged into `articles`.

**Use when:**
- Combining local + external data
- Multiple content sources
- Different directories for different formats

---

### Glob Patterns

Use glob patterns for flexible matching.

```python
from typing import Annotated

# Match multiple directories
articles: Annotated[
    list[dict],
    "app/content/{blog,posts,articles}/*.md"
]

# Match multiple extensions
articles: Annotated[
    list[dict],
    "app/content/articles/*.{md,yaml,json}"
]

# Recursive search
articles: Annotated[
    list[dict],
    "app/content/**/*.md"
]

# Combined
articles: Annotated[
    list[dict],
    "app/content/{blog,articles}/**/*.{md,yaml}"
]
```

**Glob syntax:**
- `*` - Any characters
- `**` - Any depth (recursive)
- `{a,b}` - Either `a` or `b`
- `?` - Single character

---

### Type-Safe with Dataclasses

Add validation and IDE autocomplete.

```python
from dataclasses import dataclass
from datetime import date

@dataclass
class Article:
    title: str
    slug: str
    date: date
    content: str  # Raw markdown
    html: str     # Rendered HTML

articles: list[Article]
```

**Auto-discovery still works.** Files are loaded and validated against the dataclass.

**Build fails if validation fails.** The CLI shows helpful error messages:

```
Error: app/content/articles/intro.md
  Missing required field: 'title'
  Invalid field 'date': expected date, got "not-a-date"

Build failed. Fix the errors above and try again.
```

Fix the file. Build again. Errors disappear.

**Access via attributes:**

```python
article = articles[0]

t"""
<h1>{article.title}</h1>
<time>{article.date}</time>
<div>{article.html}</div>
"""
```

**Path from `Meta`:**

```python
@dataclass
class Article:
    title: str

    class Meta:
        path = "app/content/articles/*.md"

articles: list[Article]
```

**Use `Meta.path` to override auto-discovery.**

---

### Dataclass + Annotated

Combine type safety with explicit paths.

```python
from typing import Annotated
from dataclasses import dataclass

@dataclass
class Article:
    title: str
    slug: str

articles: Annotated[
    list[Article],
    "app/content/blog/*.md",
    "data/imports/*.json"
]
```

**Result:**
- Files from both paths merged
- Each validated against `Article` dataclass

---

## File Formats

### Markdown (.md)

```markdown
---
title: "My Article"
slug: "my-article"
date: 2025-01-15
---

# Article Content

This is the **body** of the article.
```

**Frontmatter:** YAML between `---` (becomes fields)
**Body:** Everything after frontmatter

**Magic fields:**
- `content` - Raw markdown: `"# Article Content\n\nThis is the **body**..."`
- `html` - Rendered HTML: `"<h1>Article Content</h1>\n<p>This is the <strong>body</strong>..."`
- `id` - Filename without extension: `"my-article"` (from `my-article.md`)
- `title` - Auto-populated from first header if not in frontmatter (e.g., `"# Article Content"` → `"Article Content"`)

**Auto-populated title example:**

If your markdown file has no `title` in frontmatter:
```markdown
---
date: 2025-01-15
---

# Getting Started with Hyper

This is the introduction...
```

The CLI automatically sets `title = "Getting Started with Hyper"` from the first `#` header.

**With dataclass:**
```python
@dataclass
class Article:
    title: str
    date: date
    content: str  # Optional: include if you need raw markdown
    html: str     # Optional: include if you need rendered HTML

articles: list[Article]

# Access via attributes
article.content  # Raw markdown
article.html     # Rendered HTML
```

**Without dataclass:**
```python
articles: list[dict]

# All fields available via dict access
article["title"]    # "My Article"
article["content"]  # Raw markdown
article["html"]     # Rendered HTML
article["id"]       # "my-article"
```

### JSON (.json)

```json
{
  "title": "My Article",
  "slug": "my-article"
}
```

**No magic fields.** JSON has no content/html (no markdown to render).

### YAML (.yaml, .yml)

```yaml
title: My Article
slug: my-article
tags:
  - python
  - web
```

**No magic fields.** Pure data only.

### TOML (.toml)

```toml
title = "My Article"
slug = "my-article"
tags = ["python", "web"]
```

**No magic fields.** Pure data only.

**Magic fields (`content`, `html`, `id`) are only for markdown files.**

All formats work the same way. Mix them freely.

---

## Quick Reference: Content Loading

### Auto-Discovery (Simple)
```python
articles: list[dict]
```
→ `app/content/articles/**/*.{md,json,yaml,toml}`

### Single File
```python
config: dict
```
→ `app/content/config.{json,yaml,toml}`

### Explicit Path
```python
sponsors: Annotated[list[dict], "path/to/sponsors.json"]
```

### Multiple Paths
```python
articles: Annotated[
    list[dict],
    "app/content/blog/*.md",
    "data/*.json"
]
```

### Glob Patterns
```python
articles: Annotated[
    list[dict],
    "app/content/{blog,articles}/**/*.{md,yaml}"
]
```

### Type-Safe
```python
@dataclass
class Article:
    title: str

    class Meta:
        path = "app/content/articles/*.md"  # Optional

articles: list[Article]
```

### Combined (Type-Safe + Multiple Paths)
```python
@dataclass
class Article:
    title: str

articles: Annotated[
    list[Article],
    "app/content/blog/*.md",
    "data/*.json"
]
```

---

<details>
<summary><strong>Advanced: Conflict Resolution</strong></summary>

### Duplicate IDs

If two files have the same ID:

```
app/content/articles/intro.md
app/content/articles/intro.json
```

**CLI errors:**
```
Error: Duplicate ID 'intro' in 'articles' collection
Found in: intro.md, intro.json
Fix: Rename one file or use separate paths
```

### File + Directory Conflict

If both exist:
```
app/content/sponsors.json         # File
app/content/sponsors/acme.json    # Directory
```

**CLI prefers directory, warns about orphan file:**
```
Warning: Found orphan file 'sponsors.json'
Using directory: app/content/sponsors/
Remove the file or rename the directory
```

### Empty Content

If no files found:
```python
articles: list[dict]  # No app/content/articles/ directory
```

**Result:** `articles = []` (empty list, not an error)

### Validation Errors (Dataclasses)

If file doesn't match schema:
```python
@dataclass
class Article:
    title: str      # Required
    date: date      # Required

articles: list[Article]
```

**CLI errors per file:**
```
Error: app/content/articles/intro.md
  Missing required field: 'title'
  Invalid field 'date': expected date, got string "2025-13-99"
```

Build fails until all files are valid.

</details>

---

## Dynamic Routes

Generate multiple pages from one template.

### Simple: Fixed Values

Use `generate()` to specify which pages to build.

```python
# app/pages/docs/[lang].py

def generate():
    return [{"lang": lang} for lang in ['en', 'es', 'fr']]

# Injected by CLI
lang: str

# ---

t"""
<html lang={lang}>
<body>
    <h1>Documentation - {lang}</h1>
</body>
</html>
"""
```

**Build generates:**
```
dist/docs/en/index.html
dist/docs/es/index.html
dist/docs/fr/index.html
```

<details>
<summary>View generated HTML</summary>

**File `dist/docs/en/index.html`:**
```html
<html lang="en">
<body>
    <h1>Documentation - en</h1>
</body>
</html>
```
</details>

**What happens:**
1. CLI calls `generate()` at build time
2. Gets list of values: `[{"lang": "en"}, {"lang": "es"}, {"lang": "fr"}]`
3. Renders the page once for each value
4. Each render: `lang` is injected with a different value

---

### From Content

Generate pages from content automatically.

```python
# app/pages/articles/[slug].py
from app.content import articles, Article

def generate():
    for article in articles:
        yield {"slug": article.slug, "article": article}

# Injected by CLI
slug: str      # Path param (matches [slug] in filename)
article: Article  # Prop (doesn't match path, so it's a prop)

# ---

t"""
<html>
<body>
    <h1>{article.title}</h1>
    <div>{article.html}</div>
</body>
</html>
"""
```

**Build generates:**
```
dist/articles/intro/index.html
dist/articles/python-tips/index.html
```

Same output as before, but **no need to search for the article** - it's passed directly as a prop.

**What changed:**
- `generate()` yields the article in addition to the slug
- CLI detects `article` isn't a path param → it's a prop
- Props are injected automatically
- Access `article` directly (no searching needed)

**Benefits:**
- ✅ Efficient (no double iteration)
- ✅ Type-safe (IDE knows `article: Article`)
- ✅ Clean templates (direct access)

---

### Type-Safe Props (Advanced)

For complex scenarios with many props, use a dataclass.

```python
# app/pages/articles/[slug].py
from dataclasses import dataclass
from app.content import articles, Article

@dataclass
class Props:
    article: Article

def generate():
    for article in articles:
        yield {"slug": article.slug}, Props(article=article)

# Injected by CLI
slug: str
article: Article  # Auto-unwrapped from Props

# ---

t"""
<html>
<body>
    <h1>{article.title}</h1>
    <div>{article.html}</div>
</body>
</html>
"""
```

**Build generates:**
```
dist/articles/intro/index.html
dist/articles/python-tips/index.html
```

Same output. Benefits:
- ✅ Type-safe in `generate()` (IDE autocomplete when creating Props)
- ✅ Still clean in templates (props auto-unwrapped)

---

### Multiple Path Parameters

Combine multiple parameters in `generate()`.

```python
# app/pages/[lang]/articles/[slug].py
from app.content import articles, Article

def generate():
    for lang in ['en', 'es', 'fr']:
        for article in articles:
            yield {
                "lang": lang,
                "slug": article.slug,
                "article": article
            }

# Injected by CLI
lang: str
slug: str
article: Article

# ---

t"""
<html lang={lang}>
<body>
    <h1>{article.title}</h1>
</body>
</html>
"""
```

Assume 2 articles.

**Build generates:**
```
dist/en/articles/intro/index.html
dist/en/articles/python-tips/index.html
dist/es/articles/intro/index.html
dist/es/articles/python-tips/index.html
dist/fr/articles/intro/index.html
dist/fr/articles/python-tips/index.html
```

**Total:** 6 pages (3 languages × 2 articles)

<details>
<summary>View generated HTML</summary>

**File `dist/en/articles/intro/index.html`:**
```html
<html lang="en">
<body>
    <h1>Introduction</h1>
</body>
</html>
```

**File `dist/es/articles/intro/index.html`:**
```html
<html lang="es">
<body>
    <h1>Introduction</h1>
</body>
</html>
```
</details>

Every combination of `lang` × `slug`. Article passed as prop to avoid re-fetching.

---

### Filter with Custom Logic

Use `generate()` to filter or customize which pages to build.

```python
# app/pages/[lang]/articles/[slug].py
from app.content import articles, Article

def generate():
    for language in ['en', 'es', 'fr']:
        # Only published articles in this language
        filtered = [a for a in articles if a.lang == language and a.published]

        for article in filtered:
            yield {
                "lang": language,
                "slug": article.slug,
                "article": article
            }

# Injected by CLI
lang: str
slug: str
article: Article

# ---

t"""
<html lang={lang}>
<body>
    <h1>{article.title}</h1>
</body>
</html>
"""
```

Assume articles:
```python
[
    Article(slug="intro", lang="en", published=True, ...),
    Article(slug="guide", lang="en", published=False, ...),  # Not published
    Article(slug="intro", lang="es", published=True, ...),
]
```

**Build generates:**
```
dist/en/articles/intro/index.html
dist/es/articles/intro/index.html
```

**Total:** 2 pages (skipped unpublished "guide")

**Use `generate()` for:**
- All dynamic routes (required for SSG)
- Parameters that depend on each other
- Filtering (only published, only certain category, etc.)
- Custom combinations
- Props to avoid re-fetching data

---

## Complex Props

Pass multiple props for rich pages.

```python
# app/pages/articles/[slug].py
from dataclasses import dataclass
from app.content import articles, authors, Article, Author

@dataclass
class Props:
    article: Article
    author: Author
    related: list[Article]

def generate():
    for article in articles:
        # Find related data
        author = next(a for a in authors if a.id == article.author_id)
        related = [
            a for a in articles
            if a.category == article.category and a.id != article.id
        ][:3]

        yield {
            "slug": article.slug
        }, Props(
            article=article,
            author=author,
            related=related
        )

# Injected by CLI
slug: str
article: Article
author: Author
related: list[Article]

# ---

t"""
<html>
<body>
    <article>
        <h1>{article.title}</h1>
        <p>By {author.name}</p>
        <div>{article.html}</div>
    </article>

    <aside>
        <h2>Related Articles</h2>
        {[t'<a href="/articles/{a.slug}">{a.title}</a>' for a in related]}
    </aside>
</body>
</html>
"""
```

Assume data:
```python
articles = [
    Article(slug="intro", title="Introduction", category="guides", author_id="alice", ...),
    Article(slug="advanced", title="Advanced", category="guides", author_id="bob", ...),
    Article(slug="tips", title="Tips", category="guides", author_id="alice", ...),
]
authors = [
    Author(id="alice", name="Alice Smith", ...),
    Author(id="bob", name="Bob Jones", ...),
]
```

**Build generates:**
```
dist/articles/intro/index.html
dist/articles/advanced/index.html
dist/articles/tips/index.html
```

<details>
<summary>View generated HTML</summary>

**File `dist/articles/intro/index.html`:**
```html
<html>
<body>
    <article>
        <h1>Introduction</h1>
        <p>By Alice Smith</p>
        <div>...</div>
    </article>

    <aside>
        <h2>Related Articles</h2>
        <a href="/articles/advanced">Advanced</a>
        <a href="/articles/tips">Tips</a>
    </aside>
</body>
</html>
```
</details>

All props are unwrapped automatically. Access them directly.

---

## Type-Safe Params

For complex scenarios with many params, use a `Params` dataclass.

```python
# app/pages/[category]/[year]/[slug].py
from dataclasses import dataclass
from app.content import articles, Article

@dataclass
class Params:
    category: str
    year: int
    slug: str

@dataclass
class Props:
    article: Article

def generate():
    for article in articles:
        yield Params(
            category=article.category,
            year=article.date.year,
            slug=article.slug
        ), Props(article=article)

# Injected by CLI
category: str
year: int
slug: str
article: Article

# ---

t"""
<html>
<body>
    <nav>
        <a href="/{category}">All {category}</a> /
        <a href="/{category}/{year}">Year {year}</a>
    </nav>

    <h1>{article.title}</h1>
</body>
</html>
"""
```

Assume data:
```python
articles = [
    Article(slug="intro", category="guides", date=date(2025, 1, 15), title="Introduction", ...),
    Article(slug="tips", category="tutorials", date=date(2024, 3, 20), title="Tips", ...),
]
```

**Build generates:**
```
dist/guides/2025/intro/index.html
dist/tutorials/2024/tips/index.html
```

<details>
<summary>View generated HTML</summary>

**File `dist/guides/2025/intro/index.html`:**
```html
<html>
<body>
    <nav>
        <a href="/guides">All guides</a> /
        <a href="/guides/2025">Year 2025</a>
    </nav>

    <h1>Introduction</h1>
</body>
</html>
```
</details>

**Benefits:**
- ✅ Type-safe param construction in `generate()`
- ✅ IDE autocomplete for all params
- ✅ Clean separation of concerns

**When to use:**
- 3+ path parameters
- Complex param logic
- Type safety matters

---

## Pagination

Split large collections into multiple pages.

### Setup: Import from Hyper

Import pagination helpers from `hyper`.

```python
from hyper import Page, paginate
```

**The `Page` type is shipped with Hyper:**

```python
@dataclass
class Page(Generic[T]):
    data: list[T]           # Current page's items

    start: int              # Index of first item (0-based)
    end: int                # Index of last item (0-based)
    total: int              # Total number of items

    current_page: int       # Current page number (1-based)
    size: int               # Items per page
    last_page: int          # Total number of pages

    url: PageURL            # Nested URL navigation
        # .current: str
        # .previous: str | None
        # .next: str | None
        # .first: str | None
        # .last: str | None
```

---

### Simple Pagination

Use `[page]` in the filename. Call `paginate()` in `generate()`.

```python
# app/pages/blog/[page].py
from hyper import Page, paginate
from app.content import articles, Article

def generate():
    yield from paginate(articles, page_size=10)

# Injected by CLI
page: Page[Article]

# ---

t"""
<html>
<body>
    <h1>Blog - Page {page.current_page} of {page.last_page}</h1>
    <p>Showing {page.start + 1}-{page.end + 1} of {page.total}</p>

    {[t'<article><h2>{item.title}</h2></article>' for item in page.data]}

    <nav>
        {page.url.previous and t'<a href="{page.url.previous}">← Previous</a>'}
        {page.url.next and t'<a href="{page.url.next}">Next →</a>'}
    </nav>
</body>
</html>
"""
```

Assume 25 articles.

**Build generates:**
```
dist/blog/1/index.html
dist/blog/2/index.html
dist/blog/3/index.html
```

<details>
<summary>View generated HTML</summary>

**File `dist/blog/1/index.html`:**
```html
<html>
<body>
    <h1>Blog - Page 1 of 3</h1>
    <p>Showing 1-10 of 25</p>

    <article><h2>First Article</h2></article>
    <!-- 9 more articles -->

    <nav>
        <a href="/blog/2">Next →</a>
    </nav>
</body>
</html>
```

**File `dist/blog/2/index.html`:**
```html
<html>
<body>
    <h1>Blog - Page 2 of 3</h1>
    <p>Showing 11-20 of 25</p>

    <!-- 10 articles -->

    <nav>
        <a href="/blog/1">← Previous</a>
        <a href="/blog/3">Next →</a>
    </nav>
</body>
</html>
```

**File `dist/blog/3/index.html`:**
```html
<html>
<body>
    <h1>Blog - Page 3 of 3</h1>
    <p>Showing 21-25 of 25</p>

    <!-- 5 articles -->

    <nav>
        <a href="/blog/2">← Previous</a>
    </nav>
</body>
</html>
```
</details>

**What happens:**
1. CLI sees `[page]` in filename → pagination mode
2. You call `paginate(data, page_size=10)` in `generate()`
3. CLI generates 3 pages (25 items ÷ 10 per page)
4. Each page: CLI injects `page: Page[Article]` with all metadata

---

### Pagination with Filtering

Filter data before paginating.

```python
# app/pages/blog/[page].py
from hyper import Page, paginate
from app.content import articles, Article

def generate():
    # Filter published articles only
    published = [a for a in articles if a.published]
    yield from paginate(published, page_size=10)

# Injected by CLI
page: Page[Article]

# ---

t"""
<h1>Page {page.current_page}</h1>
{[t'<article>{item.title}</article>' for item in page.data]}
"""
```

Assume 15 published articles (out of 25 total).

**Build generates:**
```
dist/blog/1/index.html
dist/blog/2/index.html
```

Only 2 pages (15 published ÷ 10 per page).

---

### Navigation Links

Use `page.url` for all navigation.

```python
# app/pages/blog/[page].py
from hyper import Page, paginate
from app.content import articles, Article

def generate():
    yield from paginate(articles, page_size=10)

page: Page[Article]

# ---

t"""
<nav>
    <!-- First/Last always available -->
    <a href="{page.url.first}">First</a>

    <!-- Previous/Next are None on first/last pages -->
    {page.url.previous and t'<a href="{page.url.previous}">← Previous</a>'}

    <span>Page {page.current_page} of {page.last_page}</span>

    {page.url.next and t'<a href="{page.url.next}">Next →</a>'}

    <a href="{page.url.last}">Last</a>
</nav>
"""
```

**URLs generated:**
- `page.url.current` → `/blog/2`
- `page.url.previous` → `/blog/1` (or `None` on page 1)
- `page.url.next` → `/blog/3` (or `None` on last page)
- `page.url.first` → `/blog/1`
- `page.url.last` → `/blog/3`

---

### Nested Pagination

Combine pagination with other path parameters.

```python
# app/pages/blog/[tag]/[page].py
from hyper import Page, paginate
from app.content import articles, Article

def generate():
    for tag in ["python", "javascript", "rust"]:
        filtered = [a for a in articles if tag in a.tags]
        yield from paginate(filtered, page_size=10, tag=tag)

# Injected by CLI
tag: str
page: Page[Article]

# ---

t"""
<html>
<body>
    <h1>{tag.title()} Articles - Page {page.current_page} of {page.last_page}</h1>

    {[t'<article><h2>{item.title}</h2></article>' for item in page.data]}

    <nav>
        {page.url.previous and t'<a href="{page.url.previous}">← Previous</a>'}
        {page.url.next and t'<a href="{page.url.next}">Next →</a>'}
    </nav>
</body>
</html>
"""
```

Assume:
- 25 articles tagged "python"
- 8 articles tagged "javascript"
- 3 articles tagged "rust"

**Build generates:**
```
dist/blog/python/1/index.html
dist/blog/python/2/index.html
dist/blog/python/3/index.html
dist/blog/javascript/1/index.html
dist/blog/rust/1/index.html
```

**Total:** 5 pages (3 for python, 1 for javascript, 1 for rust)

<details>
<summary>View generated HTML</summary>

**File `dist/blog/python/1/index.html`:**
```html
<html>
<body>
    <h1>Python Articles - Page 1 of 3</h1>

    <article><h2>Python Basics</h2></article>
    <!-- 9 more python articles -->

    <nav>
        <a href="/blog/python/2">Next →</a>
    </nav>
</body>
</html>
```

**File `dist/blog/python/2/index.html`:**
```html
<html>
<body>
    <h1>Python Articles - Page 2 of 3</h1>

    <!-- 10 python articles -->

    <nav>
        <a href="/blog/python/1">← Previous</a>
        <a href="/blog/python/3">Next →</a>
    </nav>
</body>
</html>
```

**File `dist/blog/javascript/1/index.html`:**
```html
<html>
<body>
    <h1>Javascript Articles - Page 1 of 1</h1>

    <!-- 8 javascript articles -->

    <!-- No navigation (only 1 page) -->
</body>
</html>
```
</details>

**What changed:**
- Added `tag=tag` to `paginate()` call
- CLI detects `tag` matches `[tag]` in filename → it's a path param
- URLs include the tag: `/blog/python/1`, `/blog/python/2`, etc.

---

### Pagination with Props

Pass extra data alongside pagination.

```python
# app/pages/authors/[author_id]/posts/[page].py
from hyper import Page, paginate
from app.content import authors, posts, Author, Post

def generate():
    for author in authors:
        author_posts = [p for p in posts if p.author_id == author.id]
        yield from paginate(
            author_posts,
            page_size=5,
            author_id=author.id,  # Matches [author_id] → param
            author=author          # Doesn't match path → prop
        )

# Injected by CLI
author_id: str
author: Author     # Prop (extra data)
page: Page[Post]

# ---

t"""
<html>
<body>
    <header>
        <img src="{author.avatar}" alt="{author.name}">
        <h1>{author.name}'s Posts</h1>
        <p>{author.bio}</p>
    </header>

    <main>
        <h2>Page {page.current_page} of {page.last_page}</h2>
        {[t'<article><h3>{post.title}</h3></article>' for post in page.data]}
    </main>

    <nav>
        {page.url.previous and t'<a href="{page.url.previous}">← Previous</a>'}
        {page.url.next and t'<a href="{page.url.next}">Next →</a>'}
    </nav>
</body>
</html>
"""
```

Assume:
- Author "alice" (ID: 1) has 12 posts
- Author "bob" (ID: 2) has 3 posts

**Build generates:**
```
dist/authors/1/posts/1/index.html
dist/authors/1/posts/2/index.html
dist/authors/1/posts/3/index.html
dist/authors/2/posts/1/index.html
```

<details>
<summary>View generated HTML</summary>

**File `dist/authors/1/posts/1/index.html`:**
```html
<html>
<body>
    <header>
        <img src="/avatars/alice.jpg" alt="Alice Smith">
        <h1>Alice Smith's Posts</h1>
        <p>Python developer and tech writer</p>
    </header>

    <main>
        <h2>Page 1 of 3</h2>
        <article><h3>Getting Started with Python</h3></article>
        <!-- 4 more posts -->
    </main>

    <nav>
        <a href="/authors/1/posts/2">Next →</a>
    </nav>
</body>
</html>
```
</details>

**Key insight:**
- `author_id=author.id` → Path parameter (matches `[author_id]`)
- `author=author` → Prop (doesn't match path, extra data)
- Both injected as top-level variables
- Same mental model as regular `generate()`

---

### Paginate Function Reference

The `paginate()` function signature:

```python
def paginate(
    items: list[T],
    page_size: int = 10,
    **kwargs
) -> Iterator[dict]:
    """
    Paginate a list of items into pages.

    Args:
        items: List of items to paginate
        page_size: Number of items per page (default: 10)
        **kwargs: Path params and props
            - Keys matching [param] in filename → path parameters
            - Other keys → props (extra data)

    Yields:
        Dict with pagination data for each page
    """
```

**Usage:**

```python
from hyper import paginate

def generate():
    # Simple
    yield from paginate(articles, page_size=10)

    # With params
    yield from paginate(articles, page_size=10, tag="python")

    # With params and props
    yield from paginate(posts, page_size=5, author_id=author.id, author=author)
```

---

## Rest Parameters

Capture multi-segment paths with `[...path]`.

```python
# app/pages/docs/[...path].py

def generate():
    return [
        {"path": "guide/intro"},
        {"path": "guide/advanced"},
        {"path": "api/reference"},
        {"path": "api/examples"}
    ]

# Injected by CLI
path: str

# ---

t"""
<html>
<body>
    <h1>Docs: {path}</h1>
</body>
</html>
"""
```

**Build generates:**
```
dist/docs/guide/intro/index.html
dist/docs/guide/advanced/index.html
dist/docs/api/reference/index.html
dist/docs/api/examples/index.html
```

<details>
<summary>View generated HTML</summary>

**File `dist/docs/guide/intro/index.html`:**
```html
<html>
<body>
    <h1>Docs: guide/intro</h1>
</body>
</html>
```
</details>

The `path` variable contains the full multi-segment string. The CLI creates nested directories based on the slashes.

---

## Quick Reference

### Two Ways to Generate Pages

**1. From content (dict-based):**
```python
def generate():
    return [{"slug": a.slug, "article": a} for a in articles]

slug: str
article: Article  # Auto-injected prop
```

**2. Type-safe (dataclass):**
```python
@dataclass
class Props:
    article: Article

def generate():
    for article in articles:
        yield {"slug": article.slug}, Props(article=article)

slug: str
article: Article  # Auto-unwrapped from Props
```

### Props Auto-Unwrapping

Props dataclass fields are injected as top-level variables.

**Don't write:**
```python
props: Props
t"""<h1>{props.article.title}</h1>"""  # ❌ Verbose
```

**Write:**
```python
article: Article  # Auto-unwrapped from Props
t"""<h1>{article.title}</h1>"""  # ✅ Clean
```

### Params vs Props Detection

CLI determines if a variable is a param or prop:

```python
def generate():
    yield {"slug": "intro", "article": article_obj}

slug: str      # Matches [slug] in filename → param
article: Article  # Doesn't match → prop
```

### Type-Safe Params (Optional)

For complex scenarios, use `Params` dataclass.

```python
@dataclass
class Params:
    category: str
    year: int
    slug: str

@dataclass
class Props:
    article: Article

def generate():
    yield Params(...), Props(...)

# All unwrapped
category: str
year: int
slug: str
article: Article
```

---

## Build Commands

### Development Server

Start the development server.

```bash
hyper dev
```

The server rebuilds pages automatically when files change.

### Build Static Site

Generate all HTML files.

```bash
hyper build
```

Output goes to the `dist/` directory.

### Preview Build

Preview the built site locally.

```bash
hyper preview
```

Opens a local server serving files from `dist/`.

---

## Deployment

Build your site.

```bash
hyper build
```

Upload the `dist/` folder to any static host:
- Netlify
- Vercel
- Cloudflare Pages
- GitHub Pages
- Any static file host

---

## Key Points

- Every page file generates one or more HTML files
- Use `Literal` for simple static values
- Use `generate()` to create pages from content
- Pass props by including them in `generate()` yield
- Props are auto-injected (CLI detects non-path variables)
- Props dataclass fields are auto-unwrapped
- Use `Params` dataclass for complex type-safe scenarios
- Build with `hyper build`
- Deploy `dist/` folder anywhere

---

**[← Previous: Streaming & SSE](07-streaming.md)** | **[Back to Index](README.md)** | **[Next: Content Collections →](09-content.md)**
