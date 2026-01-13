# Static Site Generation

> **Status**: 🚧 In active development

Generate static HTML files at build time from templates.

---

## Basic Concept

SSG runs templates once during build. Output is static HTML.

Create a template:

```hyper
<!doctype html>
<html>
<head>
    <title>About Us</title>
</head>
<body>
    <h1>About</h1>
    <p>We build great software.</p>
</body>
</html>
```

Generate static HTML:

```bash
hyper generate pages/about.hyper --static
```

Output: `dist/about/index.html`

---

## Multiple Pages from One Template

Use `generate()` to create multiple pages.

```hyper
from content import posts

def generate() -> list[Template]:
    return [
        Template(slug=post.slug, title=post.title, content=post.html)
        for post in posts
    ]

slug: str
title: str
content: str

---

<!doctype html>
<html>
<head>
    <title>{title}</title>
</head>
<body>
    <article>
        <h1>{title}</h1>
        <div>{safe(content)}</div>
    </article>
</body>
</html>
```

**Generates**:
```
dist/
├── intro/index.html
├── tutorial/index.html
└── guide/index.html
```

The `slug` determines the path. Other props customize the page.

---

## Type Safety

The `Template` function is your compiled template. It has the exact signature you defined:

```python
def Template(slug: str, title: str, content: str) -> str:
    ...
```

So this is type-checked:

```python
def generate() -> list[Template]:
    return [
        Template(slug=post.slug, title=post.title, content=post.html)
        #        ^^^ Your editor knows these parameters
    ]
```

Wrong parameters fail at type-check time, not build time.

---

## Dynamic Routes

Use props to determine the URL structure:

```hyper
def generate() -> list[Template]:
    for lang in ["en", "es", "fr"]:
        for post in posts:
            yield Template(
                lang=lang,
                slug=post.slug,
                title=post.translations[lang].title
            )

lang: str
slug: str
title: str

---

<h1>{title}</h1>
```

**Generates**:
```
dist/
├── en/
│   ├── intro/index.html
│   └── tutorial/index.html
├── es/
│   ├── intro/index.html
│   └── tutorial/index.html
└── fr/
    ├── intro/index.html
    └── tutorial/index.html
```

URL structure follows the props that determine uniqueness.

---

## Development Server

Run a development server with live rebuild:

```bash
hyper dev pages/
```

Watches for changes and regenerates pages automatically.

---

## Deployment

Generate your site:

```bash
hyper generate pages/ --static
```

Upload `dist/` to any static host:
- Netlify
- Vercel
- Cloudflare Pages
- GitHub Pages
- Any file server

No runtime required.

---

**See Also**:
- [Content](content.md) - Loading structured data
- [Templates](templates.md) - Template syntax
