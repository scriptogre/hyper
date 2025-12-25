# Static Site Generation

Generate static HTML files at build time.

***

# What is SSG?

SSG runs your code once during build. Output is static HTML.

Use SSG when:
- Content doesn't change per request
- No user sessions needed
- Maximum performance required
- Deploy to any static host

***

# Your First Static Page

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

**Output:** `dist/about/index.html`

Every file in `app/pages` becomes a static HTML file.

***

# The Generate Function

Use `generate()` to create multiple pages from one template.

## Simple Example

```python
# app/pages/docs/[lang].py

def generate():
    return [
        {"lang": "en"},
        {"lang": "es"},
        {"lang": "fr"}
    ]

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

The `generate()` function returns a list of params. CLI renders the page once for each set of params.

## From Content

```python
# app/pages/blog/[slug].py
from app.content import posts

def generate():
    for post in posts:
        yield {"slug": post.slug, "post": post}

# Injected by CLI
slug: str
post: Post

# ---

t"""
<article>
    <h1>{post.title}</h1>
    <div>{post.html}</div>
</article>
"""
```

Pass content as props. Avoid re-fetching in the template.

## Multiple Parameters

```python
# app/pages/[lang]/blog/[slug].py
from app.content import posts

def generate():
    for lang in ["en", "es", "fr"]:
        for post in posts:
            yield {
                "lang": lang,
                "slug": post.slug,
                "post": post
            }

# Injected by CLI
lang: str
slug: str
post: Post
```

Generates every combination of `lang` × `slug`.

## Type-Safe Props

For complex scenarios, use a dataclass.

```python
# app/pages/blog/[slug].py
from dataclasses import dataclass
from app.content import posts

@dataclass
class Props:
    post: Post
    related: list[Post]

def generate():
    for post in posts:
        related = [p for p in posts if p.category == post.category][:3]
        yield {"slug": post.slug}, Props(post=post, related=related)

# Injected by CLI
slug: str
post: Post
related: list[Post]  # Auto-unwrapped from Props
```

Props are unwrapped to top-level variables.

***

# Pagination

Split large collections into multiple pages.

## Simple Pagination

Import from Hyper:

```python
from hyper import Page, paginate
```

Use `[page]` in filename:

```python
# app/pages/blog/[page].py
from hyper import Page, paginate
from app.content import posts

def generate():
    yield from paginate(posts, page_size=10)

# Injected by CLI
page: Page[Post]

# ---

t"""
<h1>Blog - Page {page.current_page} of {page.last_page}</h1>
<p>Showing {page.start + 1}-{page.end + 1} of {page.total}</p>

{[t'<article>{post.title}</article>' for post in page.data]}

<nav>
    {page.url.previous and t'<a href="{page.url.previous}">← Previous</a>'}
    {page.url.next and t'<a href="{page.url.next}">Next →</a>'}
</nav>
"""
```

## Page Type

```python
@dataclass
class Page(Generic[T]):
    data: list[T]       # Current page's items
    current_page: int   # 1-based
    last_page: int      # Total pages
    start: int          # First item index (0-based)
    end: int            # Last item index (0-based)
    total: int          # Total items
    url: PageURL        # .current, .previous, .next, .first, .last
```

## Nested Pagination

Combine with other parameters:

```python
# app/pages/blog/[category]/[page].py
from hyper import Page, paginate
from app.content import posts

def generate():
    for category in ["python", "javascript", "rust"]:
        filtered = [p for p in posts if p.category == category]
        yield from paginate(filtered, page_size=10, category=category)

# Injected by CLI
category: str
page: Page[Post]

# ---

t"""
<h1>{category.title()} - Page {page.current_page}</h1>
{[t'<article>{post.title}</article>' for post in page.data]}
"""
```

Pass category to `paginate()`. URLs include the category.

## With Props

Pass extra data alongside pagination:

```python
# app/pages/authors/[author_id]/posts/[page].py
from hyper import Page, paginate
from app.content import authors, posts

def generate():
    for author in authors:
        author_posts = [p for p in posts if p.author_id == author.id]
        yield from paginate(
            author_posts,
            page_size=5,
            author_id=author.id,  # Path param
            author=author          # Prop
        )

# Injected by CLI
author_id: str
author: Author
page: Page[Post]

# ---

t"""
<header>
    <h1>{author.name}'s Posts</h1>
</header>

<main>
    {[t'<article>{post.title}</article>' for post in page.data]}
</main>

<nav>
    {page.url.previous and t'<a href="{page.url.previous}">← Previous</a>'}
    {page.url.next and t'<a href="{page.url.next}">Next →</a>'}
</nav>
"""
```

***

# Build Commands

## Development Server

```bash
hyper dev
```

Rebuilds pages when files change.

## Build Static Site

```bash
hyper build
```

Output goes to `dist/` directory.

## Preview Build

```bash
hyper preview
```

Serves files from `dist/`.

***

# Deployment

Build your site:

```bash
hyper build
```

Upload `dist/` folder to any static host:
- Netlify
- Vercel
- Cloudflare Pages
- GitHub Pages
- Any static file host

No server required. Just upload and go.

***

# Key Concepts

## Generate Function

- Returns params for each page to build
- CLI renders template once per param set
- Use `yield` for memory efficiency
- Pass props to avoid re-fetching data

## Props vs Params

```python
def generate():
    yield {"slug": "intro", "post": post_obj}

slug: str    # Matches [slug] in filename → param
post: Post   # Doesn't match → prop
```

Params determine the URL. Props are extra data.

## Pagination

- Use `paginate()` to split collections
- Returns `Page[T]` with navigation URLs
- Combine with other params
- Pass props for extra data

---

**[← Previous: Templates](templates.md)** | **[Next: Routing →](routing.md)**
