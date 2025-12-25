# Error Message Standard

Write error messages that **show, not tell**.

## Principles

1. **Simple, imperative sentences** - "Must be X" not "Only supports X"
2. **Show the actual code** - Display what the user wrote
3. **Show the fix** - Provide concrete, copy-paste ready solutions
4. **No jargon** - Use plain language
5. **Be specific** - Show exact examples, not generic descriptions

## Template

```
[Brief imperative statement of the problem]

  You wrote:
    [exact code that caused the error]

  [Solution approach 1]:
    [corrected code]

  [Solution approach 2]:
    [alternative corrected code]
```

## Example

**Bad:**
```
TemplateSyntaxError: Shorthand attribute syntax only supports simple identifiers, not complex expressions like {obj.attr}
```

**Good:**
```
Shorthand attributes must be simple variable names.

  You wrote:
    <{Component} {obj.attr}>

  Use the explicit syntax instead:
    <{Component} attr={obj.attr}>

  Or extract to a variable:
    attr = obj.attr
    <{Component} {attr}>
```

## Inspiration

Follow the **Rust/Elm compiler** standard: errors should teach users how to fix them immediately.
