# Rust Backend

Hyper's compiler already parses `.hyper` → AST in Rust. A second codegen backend would emit `.rs` files instead of `.py` files, producing server-side Rust components with the same clean syntax.

## Syntax

Rust types and braces, but no boilerplate. The `---` separator, `<{Component}>` syntax, and `{...}` slots carry over from Python.

### Props

```hyper
title: String
active: bool = false

---

<div class="card">
    if active {
        <span class="badge">Active</span>
    }
    <h1>{title}</h1>
</div>
```

All props are implicitly public. No `pub` keyword — everything above `---` is the component's API, everything below is private.

### Control Flow

Standard Rust syntax — `if/else`, `for..in`, `match`, `if let`:

```hyper
status: Option<String>

---

match status {
    Some(s) => <p>Status: {s}</p>,
    None => <p>Loading...</p>,
}
```

### Slots

```hyper
title: String

---

<html>
<head><title>{title}</title></head>
<body>{...}</body>
</html>
```

Usage:

```hyper
use crate::layouts::Page;

---

<{Page} title="Home">
    <h1>Welcome</h1>
</{Page}>
```

### Implicit Conversions

The compiler handles two things the user shouldn't have to think about:

1. **String literals** — `title="Home"` compiles to `title: "Home".into()`. No `.to_string()`.
2. **Variable references** — `{name}` compiles to `&self.name`. No explicit `&`.

## Codegen Target

Each `.hyper` file compiles to a struct implementing `std::fmt::Display`:

```rust
pub struct Card {
    pub title: String,
    pub active: bool,
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<div class=\"card\">")?;
        if self.active {
            f.write_str("<span class=\"badge\">Active</span>")?;
        }
        write!(f, "<h1>{}</h1>", &self.title)?;
        f.write_str("</div>")?;
        Ok(())
    }
}
```

Usage in Rust code:

```rust
let html = Card { title: "Welcome".into(), active: true }.to_string();
```

## Slots Strategy

Start with the **buffer strategy**: render children to a `String`, pass as a field. Simple to implement, still fast.

The **generic strategy** (`Layout<T: Display>`) is faster (zero intermediate allocation) but significantly complicates the compiler. Optimize later if needed.

## IDE Support

Same architecture as the Python backend: source maps between `.hyper` and the generated `.rs` file.

The IDE extension wraps `.hyper` content in `hyper_component! { ... }` in memory and feeds it to rust-analyzer. Span mapping translates diagnostics back to the source file. The user never sees the macro wrapper.

## Not Leptos

Leptos solves client-side reactivity (signals, closures, DOM diffing). Hyper-rs is pure server-side rendering — read data once, write HTML, done. No `Signal<T>`, no `move ||`, no `.get()`. That's why it's simpler.

## Integration

A `build.rs` script runs the compiler at build time. It scans `src/components/*.hyper`, generates `.rs` files in `OUT_DIR`, and `include!()` pulls them into the crate. Templates recompile automatically on `cargo build`.
