# Language Injection Notes

## Goal

In `.hyper` files, we want:
1. HTML syntax highlighting for HTML content
2. Python expression resolution (go-to-definition) for `{expressions}`

This works in native `.py` files with t-strings. We want the same for `.hyper`.

## Current State

Using Approach 2: HTML injection into HyperHtmlLine.
- HTML is highlighted
- Expressions don't resolve (go-to-definition doesn't work)

## Approaches Tried

### Approach 1: Python injection only

Inject Python for the entire file. HTML lines become t-strings.

```kotlin
// HyperLanguageInjector injects into HyperRootElement
// HyperHtmlInjector disabled
```

Result:
- Expressions resolve correctly
- HTML shows as green Python strings (no HTML highlighting)

### Approach 2: HTML injection into HyperHtmlLine (CURRENT)

Inject HTML into child elements. Python injection targets parent.

```kotlin
// HyperHtmlInjector targets HyperHtmlLine
// HyperLanguageInjector targets HyperRootElement
```

Result:
- HTML is highlighted
- Expressions don't resolve

### Approach 3: Both injectors on HyperRootElement (overlapping)

Both inject into the same host. Python covers entire HTML lines. HTML covers parts excluding expressions.

```
Python range: [11, 42]  // entire line
HTML ranges:  [11, 29] and [36, 42]  // overlapping
```

Result:
- Inconsistent behavior
- Sometimes expressions worked, sometimes HTML worked

### Approach 4: Non-overlapping ranges

Transpiler generates separate pieces. No range overlaps.

```
Python: [0,10] for "title: str"
Python: [29,36] for "{title}"
HTML:   [11,29] for "<div class='card'>"
HTML:   [36,42] for "</div>"
```

Result:
- Expressions show "Unresolved reference 'title'"
- HTML shows as white text (no highlighting)

The `{title}` piece creates virtual Python `t"""{title}"""`. This is separate from the `title: str` context. Python can't resolve the variable.

## Key Files

```
HyperLanguageInjector.kt  - Python injection into HyperRootElement
HyperHtmlInjector.kt      - HTML injection into HyperHtmlLine
HyperHtmlLineMixin.kt     - Implements PsiLanguageInjectionHost
rust/transpiler/src/lib.rs - Generates python_pieces for injection
```

## APIs Found in Research

```
MultiHostInjector           - Low-level, currently using
LanguageInjectionContributor - Specifies what to inject
LanguageInjectionPerformer   - How to inject (for interpolation)
PyFStringsInjector          - Python's f-string injection
```

## Open Questions

1. How does PyCharm inject HTML into t-strings while keeping expressions working?
2. Can we examine `PyFStringsInjector` source code?
3. Can we create a single Python context that includes all pieces?
4. What does `LanguageInjectionPerformer.performInjection()` allow?
