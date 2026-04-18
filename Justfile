# Build transpiler (release)
build:
    cd {{justfile_directory()}}/rust && cargo build --release

# Compile .hyper files (debug build)
run *files:
    cargo run -q --manifest-path {{justfile_directory()}}/rust/Cargo.toml -- generate {{files}}

# Run all checks (fmt, clippy, tests)
check:
    cd {{justfile_directory()}}/rust && cargo fmt --check
    cd {{justfile_directory()}}/rust && cargo clippy -- -D warnings
    cd {{justfile_directory()}}/rust && cargo test

# Run transpiler tests
test:
    cd {{justfile_directory()}}/rust && cargo test

# Format code
fmt:
    cd {{justfile_directory()}}/rust && cargo fmt

# Auto-fix clippy warnings
fix:
    cd {{justfile_directory()}}/rust && cargo clippy --fix --allow-dirty

# Update expected test files from current output
test-accept *filter:
    cd {{justfile_directory()}}/rust && cargo run --bin accept_expected -- {{filter}}

# Release a new version
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ ! "{{version}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: version must be semver (e.g. 0.2.0), got '{{version}}'"
        exit 1
    fi
    if [ -n "$(git status --porcelain)" ]; then
        echo "Error: working tree is dirty. Commit or stash changes first."
        exit 1
    fi
    just check
    sed -i '' 's/^version = ".*"/version = "{{version}}"/' pyproject.toml rust/Cargo.toml
    cd rust && cargo check --quiet 2>/dev/null
    git add pyproject.toml rust/Cargo.toml rust/Cargo.lock
    git commit -m "Release v{{version}}"
    git tag "v{{version}}"
    echo ""
    echo "Ready to publish. Run:"
    echo "  git push && git push --tags"

# Build JetBrains plugin (builds transpiler + bundles binary + builds plugin)
build-plugin: build _bundle
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    cd "$ROOT/editors/jetbrains" && ./gradlew clean buildPlugin
    cp "$ROOT/editors/jetbrains/build/distributions"/*.zip "$ROOT/editors/jetbrains/hyper-plugin.zip"

# Run JetBrains plugin sandbox
run-plugin: build-plugin
    cd {{justfile_directory()}}/editors/jetbrains && ./gradlew runIde

# Run JetBrains plugin tests
test-plugin:
    cd {{justfile_directory()}}/rust && cargo build -q
    cd {{justfile_directory()}}/editors/jetbrains && ./gradlew test

# Bundle transpiler binary into plugin resources
[private]
_bundle:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    case "$OS" in
        darwin) OS_NAME="darwin" ;;
        linux)  OS_NAME="linux" ;;
        *)      OS_NAME="$OS" ;;
    esac
    case "$ARCH" in
        arm64|aarch64) ARCH_NAME="arm64" ;;
        x86_64|amd64)  ARCH_NAME="x64" ;;
        *)             ARCH_NAME="$ARCH" ;;
    esac
    BINARY_NAME="hyper-${OS_NAME}-${ARCH_NAME}"
    SRC="$ROOT/rust/target/release/hyper"
    DEST="$ROOT/editors/jetbrains/src/main/resources/bin/${BINARY_NAME}"
    mkdir -p "$(dirname "$DEST")"
    cp "$SRC" "$DEST"
