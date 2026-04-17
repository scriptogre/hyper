# Documentation

Hyper's documentation should read like IKEA instructions: clear, concise, and direct.

## Principles

- Short, complete sentences. One concept at a time.

### Show, don't tell

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
<button {disabled}>
\```

\```html
<!-- disabled == True -->
<button disabled>

<!-- disabled == False -->
<button>
\```

`True` renders the attribute. `False` and `None` omit it. Any other value renders as `disabled="value"`.
```

Use inline comments to complement the examples shown.

### One concept per example

Bad:

```markdown
## Props

\```hyper
name: str
count: int = 0
disabled: bool = False

---

<div class={["btn", {"active": is_active}]}>
    <h1>Hello {name}</h1>
    if count > 0:
        <p>{count} items</p>
    end
    <button {disabled}>Click</button>
</div>
\```
```

Good:

```markdown
## Props

\```hyper
name: str
count: int = 0

---

<h1>Hello {name}</h1>
<p>{count} items</p>
\```

Props are declared above the `---` delimiter. They become keyword-only arguments of the compiled function.
```

Each example should demonstrate exactly one thing. Cut everything else.

## Headings and structure

Headings describe the thing, not the action. "Slots", not "Using Slots" or "How to Use Slots".

Page content is organized so that you can look at the table of contents and understand what the page covers.

## Avoid filler

Words like "simply", "easily", "just", "of course", and "obviously" don't add information. If something were obvious, it wouldn't need documenting.
