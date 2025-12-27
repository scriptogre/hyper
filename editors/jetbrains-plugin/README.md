# Hyper JetBrains Plugin

IntelliJ Platform plugin for Hyper template files (`.hyper`).

## Features

- Syntax highlighting for Python control flow and HTML
- Code folding for `if/end`, `for/end`, `match/end` blocks
- Brace matching
- HTML comments

## Development Setup

### Prerequisites

- JDK 17+
- IntelliJ IDEA (for Grammar-Kit plugin)

### Building

1. **Generate Lexer and Parser** (first time or after grammar changes):

   Open the project in IntelliJ IDEA with the Grammar-Kit plugin installed:
   - Right-click `src/main/grammar/Hyper.flex` → Run JFlex Generator
   - Right-click `src/main/grammar/Hyper.bnf` → Generate Parser Code

2. **Build the plugin**:
   ```bash
   ./gradlew build
   ```

3. **Run in sandbox IDE**:
   ```bash
   ./gradlew runIde
   ```

4. **Build distributable**:
   ```bash
   ./gradlew buildPlugin
   ```
   The plugin ZIP will be in `build/distributions/`.

### Project Structure

```
src/main/
├── grammar/
│   ├── Hyper.bnf          # Grammar-Kit parser grammar
│   └── Hyper.flex         # JFlex lexer specification
├── gen/                    # Generated lexer/parser (not committed)
├── kotlin/com/hyper/plugin/
│   ├── HyperLanguage.kt
│   ├── HyperFileType.kt
│   ├── HyperParserDefinition.kt
│   ├── HyperSyntaxHighlighter.kt
│   ├── HyperFoldingBuilder.kt
│   ├── HyperBraceMatcher.kt
│   ├── HyperCommenter.kt
│   └── psi/               # PSI element types
└── resources/
    ├── META-INF/plugin.xml
    └── icons/hyper.svg
```

## Future Enhancements

- [ ] LSP integration with Pyright for Python intelligence
- [ ] HTML language injection for tag content
- [ ] Structure view
- [ ] Go to definition for components
- [ ] Auto-completion
