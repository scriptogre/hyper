# Hyper

A Python framework for hypermedia-driven applications. Write templates in `.hyper` syntax, compile to type-safe Python.

## Quick Example

Write a template:

```hyper
# Card.hyper
title: str
content: str = ""

<div class="card">
    <h1>{title}</h1>
    if content:
        <p>{content}</p>
    end
</div>
```

Compile to Python:

```bash
hyper generate Card.hyper
```

Use it:

```python
from Card import Card

html = Card(title="Hello", content="World")
```

## Packages

| Package | Description | Python |
|---------|-------------|--------|
| `hyper` | Runtime helpers + CLI | 3.10+ |
| `hyper-content` | Content collections (Markdown, YAML) | 3.10+ |

Install the CLI and runtime:

```bash
pip install hyper
```

For content collections:

```bash
pip install hyper-content
```

## Repository Structure

```
python/
  hyper/              Runtime + CLI
  hyper-content/      Content collections
rust/
  transpiler/         Compiles .hyper → .py
editors/
  jetbrains-plugin/   IDE support for PyCharm/IntelliJ
playground/           Examples and test cases
docs/                 Design and implementation docs
tests/                Python unit tests
```

## Getting Started

### Prerequisites

- Python 3.10+
- Rust (for building the transpiler)

### Development Setup

Clone and set up the workspace:

```bash
git clone https://github.com/user/hyper.git
cd hyper
uv sync
```

Build the transpiler:

```bash
cd rust/transpiler
cargo build --release
```

Run tests:

```bash
pytest
```

## Project Status

**Working:**
- Template compilation (`.hyper` → `.py`)
- JetBrains IDE plugin with full Python intelligence
- Content collections with Markdown support

**Planned:**
- Server-side rendering (SSR)
- Streaming responses
- File-based routing
