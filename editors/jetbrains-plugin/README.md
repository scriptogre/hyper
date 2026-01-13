# JetBrains Plugin for Hyper

IDE support for `.hyper` files in PyCharm and IntelliJ IDEA.

## Installation

### From JetBrains Marketplace (Coming Soon)

1. Open **Settings → Plugins → Marketplace**
2. Search for "Hyper"
3. Click **Install**

### From Disk

1. Build the plugin: `./gradlew buildPlugin`
2. Open **Settings → Plugins → ⚙️ → Install Plugin from Disk**
3. Select `build/distributions/hyper-plugin-*.zip`

## Features

**Syntax Highlighting**
- Python keywords and expressions
- HTML tags and attributes
- Control flow (`if/for/end` blocks)

**IDE Intelligence**
- **Go to Definition**: Click `{user.name}` to jump to where `user` is defined
- **Autocompletion**: Full Python completion inside `{expressions}`
- **Type Checking**: Real-time validation of Python expressions

**Project View**
- Generated `.py` files nest under their `.hyper` source
- Shows as: `Button.hyper` → `Button.py`

**Hyper Inspector**
- Tool window showing the transpiled Python code
- Useful for debugging template issues

## How It Works

The plugin uses **language injection** to provide IDE features:

1. Runs the Rust transpiler on each `.hyper` file
2. Creates a virtual Python representation for the IDE
3. Maps positions between `.hyper` source and virtual `.py`
4. Injects HTML highlighting into f-string regions

This leverages the IDE's existing Python and HTML support.

## Bundled Binary

The plugin includes a pre-built `hyper` binary for macOS ARM64 at:

```
src/main/resources/bin/hyper-darwin-arm64
```

For other platforms, build the transpiler and update the path in `HyperTranspilerService.kt`.

## Development

### Prerequisites

- JDK 17+
- Gradle

### Build

```bash
./gradlew buildPlugin
```

### Run in Sandbox

```bash
./gradlew runIde
```

Launches a sandboxed IDE with the plugin installed for testing.

### Project Structure

```
src/main/
  kotlin/com/hyper/plugin/
    HyperLanguage.kt           Language definition
    HyperFileType.kt           File type registration
    HyperSyntaxHighlighter.kt  Syntax coloring
    HyperLanguageInjector.kt   Python/HTML injection
    HyperTranspilerService.kt  Transpiler integration
    HyperFileListener.kt       Auto-generation on save
    HyperInspectorToolWindow.kt Debug tool window
  grammar/
    Hyper.bnf                  Parser grammar
    Hyper.flex                 Lexer specification
  resources/
    META-INF/plugin.xml        Plugin manifest
    bin/                       Bundled transpiler binary
```
