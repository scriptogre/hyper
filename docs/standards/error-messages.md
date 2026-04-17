# Error Messages

Hyper's error messages follow the Rust and Elm compiler tradition: every error should show the user what went wrong and how to fix it.

## Structure

Every error message has three parts:

1. A short, imperative statement of the problem
2. The code that caused it
3. One or more concrete fixes

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

## Style

- Imperative over descriptive: "must be X", not "only supports X"
- No internal jargon or AST terminology in user-facing messages
- Fixes should be copy-paste ready
- If there's more than one way to fix something, show both
