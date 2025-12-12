# Architecture & Implementation Plan

## Overview

Hyper is a Python web framework with two modes:
1. **SSG (Static Site Generation)** - Build static HTML at build time
2. **SSR (Server-Side Rendering)** - Render HTML on-demand *(planned)*

---

## Core Technology Decisions

### CLI: Rust

**Why Rust:**
- Fast builds (milliseconds, not seconds)
- Instant hot reload
- Single binary distribution
- Incremental build system
- Zero Python dependencies to install

**What it does:**
- File watching & hot reload
- Markdown/YAML/TOML/JSON parsing
- Content collection loading
- Build orchestration
- Dev server
- Executes Python page files

### Page Logic: Python

**Why Python:**
- User writes pages in Python
- Full stdlib + any PyPI package at build time
- Dataclasses for content schemas
- Natural for data processing

**What users write:**
- `app/pages/*.py` - Page files
- `app/content/__init__.py` - Content definitions
- Templates with t-strings (Python 3.14+)

---

## Distribution

### Via `uv tool`

Package as Python package with embedded Rust binary (like Ruff):

```bash
# Instant use:
uvx hyper init myblog
uvx hyper build
uvx hyper dev

# Or install:
uv tool install hyper
hyper build
```

**Structure:**
```
hyper/
  pyproject.toml          # Python package metadata
  hyper/
    __init__.py
    cli.py                # Thin wrapper around Rust binary
    bin/
      hyper-linux-x64     # Precompiled Rust binaries
      hyper-macos-arm64
      hyper-windows.exe
  src/                    # Rust CLI source
    main.rs
    parser/
    builder/
    watcher/
  Cargo.toml
```

---

## Build Process

### SSG Build Flow

1. **Scan** `app/content/__init__.py`
   - Find type hints like `blogs: list[Blog]`
   - Read `Meta.path` glob patterns
   - Parse content files (Rust: fast)
   - Validate against dataclass schemas

2. **Scan** `app/pages/`
   - Find all `.py` files
   - Detect dynamic routes `[param].py`
   - Check for `Literal` type hints or `generate()` functions

3. **Generate paths**
   - Evaluate `Literal[*[...]]` expressions
   - Call `generate()` functions
   - Build list of pages to render

4. **Render pages**
   - For each page path:
     - Inject content collections
     - Inject path parameters
     - Execute Python file
     - Extract t-string output
     - Write to `dist/`

5. **Incremental builds**
   - Track file dependencies
   - Only rebuild changed pages
   - Cache parsed content

### Dev Server

1. **Watch** file changes
2. **On change:**
   - Determine affected pages
   - Rebuild only those pages
   - Send reload signal to browser (WebSocket)
3. **Serve** from `dist/` with instant updates

---

## Implementation Phases

### Phase 1: Core SSG (Rust CLI)
- [ ] File watching with `notify`
- [ ] Markdown parser with `pulldown-cmark`
- [ ] YAML/TOML/JSON parsers
- [ ] Python executor (subprocess)
- [ ] Basic build system
- [ ] Dev server with hot reload

### Phase 2: Content Collections
- [ ] Parse `app/content/__init__.py`
- [ ] Load collections via glob patterns
- [ ] Inject into Python execution context
- [ ] Dataclass validation
- [ ] Support all file formats

### Phase 3: Dynamic Routes & Generation
- [ ] Parse `Literal[*[...]]` expressions
- [ ] Execute `generate()` functions
- [ ] Path parameter injection
- [ ] Generate static pages

### Phase 4: Incremental Builds
- [ ] Dependency tracking
- [ ] Selective rebuilds
- [ ] Content cache
- [ ] Fast dev iteration

### Phase 5: Polish
- [ ] CLI scaffolding (`hyper init`)
- [ ] Better error messages
- [ ] Build statistics
- [ ] Performance optimization

### Phase 6: SSR (Future)
- [ ] ASGI server mode
- [ ] Request/response handling
- [ ] Dynamic rendering
- [ ] Hybrid mode (static + dynamic)

---

## Key Design Principles

1. **Speed is non-negotiable** - Rust handles all I/O and parsing
2. **Zero config for simple cases** - `blogs: list[dict]` just works
3. **Powerful when needed** - Dataclasses, validation, full Python
4. **Python ecosystem** - Use any library at build time
5. **No runtime dependencies** - Pure static HTML output
6. **Familiar to Python devs** - Standard library, no DSLs

---

## File Structure (User Projects)

```
myblog/
  app/
    pages/
      index.py              # /
      about.py              # /about
      blog/
        index.py            # /blog
        [slug].py           # /blog/:slug
    content/
      __init__.py           # Content definitions
      blog/
        post-1.md
        post-2.md
    components/
      Header.py             # Reusable components
  public/
    styles.css
    images/
  dist/                     # Generated output (gitignored)
```

---

## Performance Targets

- **Cold build**: < 1 second for 100 pages
- **Incremental build**: < 100ms for single page change
- **Hot reload**: < 50ms from file save to browser refresh
- **Dev server startup**: < 100ms

These are achievable with Rust.

---

## Why Not Pure Python?

**Considered but rejected:**
- Python startup overhead (~50-100ms per invocation)
- GIL limits parallelization
- Slower file I/O and parsing
- Distribution requires Python installation
- Hard to achieve <100ms incremental builds

**Hybrid approach wins:**
- Rust for performance-critical paths
- Python for user code (where flexibility matters)
- Best of both worlds

---

## Next Steps

1. ✅ Design docs complete (SSG focused)
2. ⏳ Create Rust CLI skeleton
3. ⏳ Implement basic build system
4. ⏳ Add content collections
5. ⏳ Add dynamic routes
6. ⏳ Polish and optimize

---

**Status:** Planning phase - design documents complete for SSG functionality.
