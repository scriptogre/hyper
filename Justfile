# Build transpiler (release)
build:
    cd {{justfile_directory()}}/rust && cargo build --release

# Compile .hyper files (debug build)
run *files:
    cargo run -q --manifest-path {{justfile_directory()}}/rust/transpiler/Cargo.toml -- generate {{files}}

# Run transpiler tests
test:
    cd {{justfile_directory()}}/rust && cargo test

# Update expected test files from current output
test-accept *filter:
    cd {{justfile_directory()}}/rust/transpiler && cargo run --example accept_expected -- {{filter}}


# Build JetBrains plugin (builds transpiler + bundles binary + builds plugin)
build-plugin: build _bundle
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    cd "$ROOT/editors/jetbrains-plugin" && ./gradlew clean buildPlugin
    cp "$ROOT/editors/jetbrains-plugin/build/distributions"/*.zip "$ROOT/editors/jetbrains-plugin/hyper-plugin.zip"

# Run JetBrains plugin sandbox
run-plugin: build-plugin
    cd {{justfile_directory()}}/editors/jetbrains-plugin && ./gradlew runIde

# Run JetBrains plugin tests
test-plugin:
    cd {{justfile_directory()}}/rust && cargo build -q
    cd {{justfile_directory()}}/editors/jetbrains-plugin && ./gradlew test

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
    DEST="$ROOT/editors/jetbrains-plugin/src/main/resources/bin/${BINARY_NAME}"
    mkdir -p "$(dirname "$DEST")"
    cp "$SRC" "$DEST"
