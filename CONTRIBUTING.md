# Contributing

## Prerequisites

- [Rust toolchain](https://rustup.rs/)
- [uv](https://docs.astral.sh/uv/)
- [just](https://github.com/casey/just)
- JDK 17+ (only for JetBrains plugin work)

## Setup

```bash
git clone https://github.com/scriptogre/hyper.git
cd hyper
uv sync
just build
```

## Running Tests

```bash
just test              # Rust transpiler tests
pytest                  # Python runtime tests
just test plugin        # JetBrains plugin tests
```

## Linting

```bash
cd rust && cargo fmt --check
cd rust && cargo clippy -- -D warnings
```
