# Static Site Generation (SSG)

HyperProse can work as a traditional web server, a static site generator, or a hybrid of both - just like Astro!

---

## Default: Server Mode

By default, all routes are **server-rendered on demand**. This is perfect for dynamic applications with user-specific content.

```python
# routes/dashboard.py
# No marker needed - server-rendered by default

from hyperprose import Request

request: Request

user = get_current_user(request)

t"""
<html>
<body>
    <h1>Welcome, {user.name}!</h1>
    <p>Your personalized dashboard</p>
</body>
</html>
"""
```

---

## Opt-in: Static Generation

Add `prerender = True` to pre-render routes at build time:

```python
# routes/about.py
prerender = True  # Generate static HTML at build time

company = get_company_info()

t"""
<html>
<body>
    <h1>About {company.name}</h1>
    <p>{company.description}</p>
</body>
</html>
"""
```

**Build result:** `dist/about/index.html` (static file)

---

## Static Paths for Dynamic Routes

For dynamic routes like `{user_id}.py`, specify which paths to pre-render:

```python
# routes/users/{user_id}.py
prerender = True

def get_static_paths():
    """Return list of param dicts to pre-render"""
    users = get_all_users()
    return [{"user_id": str(u.id)} for u in users]

user_id: int
user = get_user(user_id)

t"""
<html>
<body>
    <h1>{user.name}</h1>
    <p>Bio: {user.bio}</p>
    <p>Email: {user.email}</p>
</body>
</html>
"""
```

**Build result:**
- `dist/users/1/index.html`
- `dist/users/2/index.html`
- `dist/users/3/index.html`
- etc.

**Key points:**
- `get_static_paths()` returns a list of dictionaries
- Each dict contains path parameter values
- All params must be strings (will be converted by type hints)

---

## Hybrid Application Example

```
routes/
  _base.py              # Layout component
  index.py              # Static homepage (prerender=True)
  about.py              # Static about page (prerender=True)
  blog/
    index.py            # Static blog list (prerender=True)
    intro.md            # Static blog post (prerender=True)
    advanced.md         # Static blog post (prerender=True)
  users/
    {user_id}.py        # Static user profiles (prerender=True)
  dashboard.py          # Dynamic dashboard (server-rendered)
  api/
    login.py            # Dynamic API endpoint (server-rendered)
  stream/
    notifications.py    # Dynamic SSE stream (server-rendered)
```

**Result:** Fast static pages where possible, dynamic where needed!

---

## Build Commands

### Development Mode

All routes server-rendered:

```bash
# Using HyperProse CLI
hyperprose dev

# Or use any ASGI server
uvicorn app:app --reload
```

### Build Static Site

Generates HTML for routes with `prerender=True`:

```bash
hyperprose build
```

### Preview Production Build

```bash
hyperprose preview
```

---

## Deployment

### Static Sites Only

Build and deploy to any static host:

```bash
hyperprose build
# Upload dist/ folder to:
# - Netlify
# - Vercel
# - Cloudflare Pages
# - GitHub Pages
# - Any static host
```

### Server-Rendered or Hybrid

Deploy to any ASGI-compatible host:

```bash
# Deploy to:
# - Fly.io
# - Railway
# - Render
# - DigitalOcean App Platform
# - Any host supporting ASGI (Uvicorn, Hypercorn, etc.)
```

**For hybrid apps:**
1. Pre-build static files: `hyperprose build`
2. Deploy the app with pre-generated `dist/` folder
3. Server serves static files from `dist/` for prerendered routes
4. Dynamic routes continue to work server-side

---

## When to Use Static vs Dynamic

### Use `prerender = True` for:
- Marketing pages (home, about, pricing)
- Blog posts and articles
- Documentation
- User profiles (if public and cacheable)
- Any content that doesn't change per-request

### Keep Server-Rendered for:
- User dashboards
- Forms and form submissions
- Authentication pages
- Real-time data
- Personalized content
- API endpoints

---

## Multiple Static Paths with Nested Routes

```python
# routes/blog/{category}/{slug}.py
prerender = True

def get_static_paths():
    """Return list of param dicts for all posts"""
    posts = get_all_blog_posts()
    return [
        {
            "category": post.category,
            "slug": post.slug
        }
        for post in posts
    ]

category: str
slug: str

post = get_post(category, slug)

t"""
<html>
<body>
    <nav>Category: {category}</nav>
    <article>
        <h1>{post.title}</h1>
        <div>{post.content:safe}</div>
    </article>
</body>
</html>
"""
```

**Build result:**
- `dist/blog/tech/python-tips/index.html`
- `dist/blog/design/color-theory/index.html`
- etc.

---

## Incremental Static Regeneration (ISR)

Coming soon! Rebuild specific pages without rebuilding the entire site.

---

## Key Points

- **Default = server-rendered (dynamic)**
- **Opt-in with `prerender = True`**
- **Use `get_static_paths()` for dynamic routes**
- **Hybrid mode = best of both worlds**
- **Deploy static to CDN or server to ASGI host**
- **Choose per-route based on needs**

---

**[← Previous: Streaming & SSE](07-streaming.md)** | **[Back to Index](README.md)** | **[Next: Markdown →](09-markdown.md)**
