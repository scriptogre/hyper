# JetBrains Editor Support Implementation

**Goal:** Provide full syntax highlighting, code completion, and navigation for `.hyper` files by leveraging existing Python and HTML support in the JetBrains ecosystem.

**Inspiration:** [Templ (Go)](https://templ.guide) and how it maps custom syntax to valid Go code for the IDE.

## The Strategy: Virtual Python File

Instead of writing a full language server or complex PSI logic from scratch, we will treat `.hyper` files as "Syntactic Sugar" for a valid Python file. The plugin's core responsibility is to **transpile** the `.hyper` content into a valid virtual Python PSI (Program Structure Interface) tree in the background.

We essentially show the user the `.hyper` source, but we ask the IDE to analyze the generated Python code.

### The Mapping

We map `.hyper` constructs to Python constructs that produce the equivalent AST/runtime behavior.

| Hyper Construct | Virtual Python Representation | Notes |
| :--- | :--- | :--- |
| `users: List[User]` | `users: List[User] = ...` | Top-level props become typed variables. |
| `if user.active:` | `if user.active:` | Control flow headers map 1:1. |
| `<div>...</div>` | `t"""<div>...</div>"""` | HTML lines become t-strings (PEP 750). |
| `end` | `pass` (with un-indent) | The crucial step: `end` triggers a dedent in the generated Python. |
| `{user.name}` | `{user.name}` | Inside the t-string, Python interpolation is preserved. |

### Example Transformation

**Source (.hyper):**
```python
user: User

<ul>
    if user.active:
        <li>{user.name}</li>
    end
</ul>
```

**Generated Virtual Python (.py):**
```python
user: User = ...

t"""<ul>"""
if user.active:
    # We enforce indentation here in the generated file
    t"""    <li>{user.name}</li>"""
    pass # Corresponds to 'end' - dedents the virtual block
t"""</ul>"""
```

## Implementation Steps

### 1. The Parser (GrammarKit)
We need a robust parser (`Hyper.bnf`) that produces a PSI tree identifying:
- **Control Headers:** `if ...:`, `for ...:`, `match ...:`
- **Blocks:** The content between a header and `end`.
- **HTML Segments:** Raw HTML text.
- **Expressions:** `{...}` blocks inside HTML.

### 2. The Transpiler (FileViewProvider)
We implement a `FileViewProvider` that creates a "Dual View":
- **Source View:** The text the user sees.
- **Generated View:** The Python text we generate.

We must implement `com.intellij.psi.impl.source.PsiFileImpl` or use `MultiplePsiFilesPerDocumentFileViewProvider`.
The key complexity is **mapping offsets**. When the user types in the `.hyper` file at offset `100`, we must translate that to offset `150` (or wherever) in the virtual Python file so completion works.

### 3. Syntax Highlighting
- **Python:** We rely on the Python plugin to highlight the generated structure. We map the colors back to the source.
- **HTML:** We rely on the Python plugin's "Language Injection" (t-string support) to highlight the string contents as HTML.

### 4. Handling `end` vs Indentation
Hyper ignores indentation; Python requires it.
Our Transpiler must **synthesize indentation**.
- When we see `if ...:`, we increase the indent level for subsequent generated lines.
- When we see `end`, we decrease the indent level.

### 5. Type Safety & Completion
Because the generated file is valid Python (with type hints), PyCharm's existing analysis engine will automatically provide:
- Auto-completion for `user.`
- Type checking (e.g., flagging if `user` doesn't have an `active` field).
- Refactoring support (Rename `user` -> `currentUser`).

## Why this approach?

1.  **Feasibility:** We don't write a Python analyzer. We use JetBrains'.
2.  **Maintenance:** As PyCharm improves (e.g., better t-string support), we get it for free.
3.  **Completeness:** We support *all* Python syntax that works inside a function body.

## Challenges

-   **Offset Mapping:** Ensuring the cursor aligns perfectly when the generated code differs in length (e.g., adding `html("""...""")` adds chars). We might need to use whitespace padding or careful mapping to keep offsets aligned where possible.
-   **Error Handling:** If the user writes broken code, the generated Python might be broken. We need to ensure error messages point to the correct line in the `.hyper` file.
