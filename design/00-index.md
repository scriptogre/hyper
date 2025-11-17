# Hyper Documentation Index

## Getting Started

- **[Architecture](00-architecture.md)** - Technical decisions and implementation plan
- **[Overview](01-overview.md)** - What is Hyper and why use it

---

## SSG (Static Site Generation)

Core documentation for building static sites.

### Fundamentals

1. **[Routing](02-routing.md)** - File-based routing, dynamic routes
2. **[Templates](03-templates.md)** - t-strings, layouts, components
3. **[Markdown](04-markdown.md)** *(to be written)* - Markdown files, frontmatter, Python execution blocks

### Static Generation

4. **[Static Site Generation](08-ssg.md)** - Building static HTML, path generation, `Literal` and `generate()`
5. **[Content Collections](09-content.md)** - Organize content, dataclasses, querying

### Advanced

6. **[Dependency Injection](05-dependency-injection.md)** - Path params, query params, headers *(needs SSG update)*

---

## SSR (Server-Side Rendering)

*(Planned - not yet implemented)*

### Server Basics

7. **[Forms & Actions](06-forms.md)** - Form handling, validation, mutations
8. **[Streaming & SSE](07-streaming.md)** - Real-time updates, server-sent events

### Advanced

9. **[Advanced Features](10-advanced.md)** - Middleware, error handling, sessions
10. **[API Reference](11-api-reference.md)** - Complete API documentation
11. **[HyperLive](12-hyperlive.md)** - Real-time reactivity

## Questions to Resolve

1. Should we split routing doc into `02-routing-ssg.md` and `02-routing-ssr.md`?
2. Should all SSG docs go in a `ssg/` subfolder?
3. Should we create a quick start guide separate from overview?
4. Do we need a "Migrating from X" guide (Hugo, Zola, Jekyll)?

---

**Next:** Create 04-markdown.md following IKEA-style documentation pattern.
