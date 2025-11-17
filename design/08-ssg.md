# Static Site Generation

Hyper generates static HTML files at build time.

---

## Your First Static Page

Create a page file:

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

Build the site:

```bash
hyper build
```

**Result:** `dist/about/index.html`

Every page file generates one HTML file. The file path becomes the URL path.

---

## Pages with Data

Load data at build time:

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

The code runs once during build. The result is static HTML.

---

## Static Paths for Dynamic Routes

Dynamic routes need to know which paths to generate at build time.

### Simple Case: Static Values

Use `Literal` to specify static values:

```python
# app/pages/docs/[lang].py
from typing import Literal

lang: Literal['en', 'es', 'fr']

t"""
<html>
<body>
    <h1>Documentation - {lang}</h1>
</body>
</html>
"""
```

**Build result:**
- `dist/docs/en/index.html`
- `dist/docs/es/index.html`
- `dist/docs/fr/index.html`

The framework reads the `Literal` type hint. It generates one page for each value.

### Dynamic Values from Content

Use `Literal` with unpacking to generate from content:

```python
# app/pages/blog/[slug].py
from typing import Literal
from app.content import posts

# Path parameters (injected by framework)
slug: Literal[*[p.slug for p in posts]]

# ---

# Use injected parameter to find the post
post = next(p for p in posts if p.slug == slug)

t"""
<html>
<body>
    <h1>{post.title}</h1>
    <div>{post.html}</div>
</body>
</html>
"""
```

**Convention:** Use `# ---` to separate injected parameters from your code.

**What happens:**
- CLI evaluates `posts` at build time
- Extracts each post's slug
- Renders this page once for each slug value
- Each render: `slug` is injected with a different value

**Build result:**
- `dist/blog/intro/index.html`
- `dist/blog/python-tips/index.html`
- `dist/blog/advanced/index.html`
- (one page per post)

### Multiple Parameters

Combine multiple `Literal` parameters:

```python
# app/pages/[lang]/blog/[slug].py
from typing import Literal
from app.content import posts

# Path parameters (injected by framework)
lang: Literal['en', 'es', 'fr']
slug: Literal[*[p.slug for p in posts]]

# ---

post = next(p for p in posts if p.slug == slug)

t"""
<html lang={lang}>
<body>
    <h1>{post.title}</h1>
</body>
</html>
"""
```

**What happens:**
- Framework generates every combination of `lang` and `slug`
- Each render gets different values injected

**Build result:**
- `dist/en/blog/intro/index.html` (lang='en', slug='intro')
- `dist/en/blog/python-tips/index.html` (lang='en', slug='python-tips')
- `dist/es/blog/intro/index.html` (lang='es', slug='intro')
- `dist/es/blog/python-tips/index.html` (lang='es', slug='python-tips')
- `dist/fr/blog/intro/index.html` (lang='fr', slug='intro')
- (etc.)

This is called a cartesian product. Every `lang` × every `slug`.

### Complex Logic: Use generate()

For filtering, conditional logic, or dependent parameters, define `generate()`:

```python
# app/pages/[lang]/blog/[slug].py
from typing import Literal
from app.content import posts

# Path parameters (injected based on generate)
lang: str
slug: str

def generate():
    """Generate paths with custom logic"""
    for language in ['en', 'es', 'fr']:
        # Filter posts by language and published status
        filtered = [p for p in posts if p.lang == language and p.published]
        for post in filtered:
            yield {"lang": language, "slug": post.slug}

# ---

post = next(p for p in posts if p.slug == slug)

t"""
<html lang={lang}>
<body>
    <h1>{post.title}</h1>
</body>
</html>
"""
```

The function returns dictionaries. Each dictionary specifies values to inject for one page render.

**Use `generate()` when you need:**
- Parameters that depend on each other
- Filtering or conditional logic
- Pagination
- Complex transformations

**Use `Literal` when:**
- Parameters are independent
- Values are static or simple to extract
- No filtering needed

### Return Values from generate()

Return a list of dictionaries:

```python
def generate():
    return [
        {"category": "tech", "slug": "python"},
        {"category": "design", "slug": "colors"},
    ]
```

Each dictionary provides parameter values for one page.

Use list comprehensions for concise code:

```python
from app.content import posts

def generate():
    return [{"slug": p.slug} for p in posts]
```

Yield dictionaries for large datasets:

```python
from app.content import posts

def generate():
    for post in posts:
        yield {"slug": post.slug}
```

Yielding is more memory-efficient than building a list.

### Pagination

Paginate content across multiple pages:

```python
# app/pages/blog/[page].py
from app.content import posts as all_posts

# Path parameter (injected by framework)
page: int

def generate():
    """Calculate how many pages we need"""
    published = [p for p in all_posts if p.published]
    page_size = 10
    total_pages = (len(published) + page_size - 1) // page_size

    return [{"page": i} for i in range(1, total_pages + 1)]

# ---

# Get posts for the current page
published = [p for p in all_posts if p.published]
page_size = 10
start = (page - 1) * page_size
page_posts = published[start:start + page_size]

t"""
<html>
<body>
    <h1>Blog - Page {page}</h1>
    {[t'<article><h2>{post.title}</h2></article>' for post in page_posts]}
</body>
</html>
"""
```

**Build result:**
- `dist/blog/1/index.html` (posts 1-10)
- `dist/blog/2/index.html` (posts 11-20)
- `dist/blog/3/index.html` (posts 21-30)
- (etc.)

### Rest Parameters

Capture multi-segment paths with `[...param]`:

```python
# app/pages/docs/[...path].py
from typing import Literal

# Path parameter (injected by framework)
path: Literal[
    'guide/intro',
    'guide/advanced',
    'api/reference',
    'api/examples'
]

# ---

t"""
<html>
<body>
    <h1>Docs: {path}</h1>
</body>
</html>
"""
```

The `[...path]` syntax captures the entire remaining URL path.

**Build result:**
- `dist/docs/guide/intro/index.html`
- `dist/docs/guide/advanced/index.html`
- `dist/docs/api/reference/index.html`
- `dist/docs/api/examples/index.html`

The `path` variable contains the full multi-segment string.

### Type Safety

`Literal` provides compile-time type checking:

```python
# Path parameter (injected by framework)
lang: Literal['en', 'es', 'fr']

# ---

# IDE knows lang can only be 'en', 'es', or 'fr'
if lang == 'de':  # ⚠️ IDE warning: 'de' not in Literal
    ...
```

Your IDE catches invalid values before you build.

For dynamic `Literal` values, the IDE treats them as `str`:

```python
from app.content import posts

slug: Literal[*[p.slug for p in posts]]
# IDE sees this as 'str' since values aren't known at type-check time
```

Static literals get full type checking. Dynamic literals work at build time only.


---

## Build Commands

### Development Server

Start the development server:

```bash
hyper dev
```

The server rebuilds pages automatically when files change.

### Build Static Site

Generate all HTML files:

```bash
hyper build
```

Output goes to the `dist/` directory.

### Preview Build

Preview the built site locally:

```bash
hyper preview
```

Opens a local server serving files from `dist/`.

---

## Deployment

Build your site:

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

## Server-Side Rendering (SSR)

**Coming soon.** Support for on-demand rendering is planned.

Dynamic routes will work without `prerender = True`. The framework will render pages on each request.

---

## Hybrid Mode

**Coming soon.** Mix static and dynamic routes in the same app.

Static pages will be pre-built. Dynamic pages will render on-demand.

---

## Key Points

- **All routes are static by default**
- **Use `Literal` for simple path generation**
- **Use `generate()` for complex cases**
- **Build with `hyper build`**
- **Deploy `dist/` folder to any static host**

---

**[← Previous: Streaming & SSE](07-streaming.md)** | **[Back to Index](README.md)** | **[Next: Markdown →](09-markdown.md)**
