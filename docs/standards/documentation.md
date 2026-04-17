# Documentation

Hyper's documentation should read like IKEA instructions: clear, concise, and direct.

## Principles

- Short, complete sentences. One concept at a time.
- Lead with code examples. Explain after, and only when needed.
- Each example should demonstrate exactly one thing. Cut everything else.

## Show, then tell

Bad:

```markdown
## Boolean Attributes

Boolean attributes in Hyper work similarly to how they work in HTML.
When you pass a boolean value to an attribute, Hyper will conditionally
render the attribute based on the truthiness of the value. If the value
is `True`, the attribute is rendered without a value. If `False`, the
attribute is omitted entirely.
```

Good:

```markdown
## Boolean Attributes

\```hyper
<button {disabled}>Click</button>
\```

\```html
<!-- disabled=True -->
<button disabled>Click</button>

<!-- disabled=False -->
<button>Click</button>
\```
```

The HTML comments do the explaining. No extra prose needed.

## Headings and structure

Headings describe the thing, not the action. "Slots", not "Using Slots" or "How to Use Slots".

Page content is organized so that you can look at the table of contents and understand what the page covers.

## Avoid filler

Words like "simply", "easily", "just", "of course", and "obviously" don't add information. If something were obvious, it wouldn't need documenting.
