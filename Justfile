# Default: build transpiler, bundle it in plugin, and build plugin
default: build

# Build transpiler
build-transpiler:
    cd {{justfile_directory()}}/rust && cargo build --release

# Build plugin (always rebuilds + bundles transpiler first)
build-plugin: build-transpiler _bundle
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    echo "Building JetBrains plugin..."
    cd "$ROOT/editors/jetbrains-plugin" && ./gradlew clean buildPlugin
    cp "$ROOT/editors/jetbrains-plugin/build/distributions"/*.zip "$ROOT/editors/jetbrains-plugin/hyper-plugin.zip"
    echo ""
    echo "✅ Plugin built!"
    echo "📦 Install: editors/jetbrains-plugin/hyper-plugin.zip"

# Build everything (or a specific target)
build target="":
    #!/usr/bin/env bash
    set -e
    case "{{target}}" in
        transpiler) just build-transpiler ;;
        plugin)     just build-plugin ;;
        "")         just build-transpiler && just _bundle && just build-plugin ;;
        *)          echo "Usage: just build [transpiler|plugin]"; exit 1 ;;
    esac

# Run transpiler or plugin
run target *args:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"

    case "{{target}}" in
        transpiler)
            "$ROOT/rust/target/release/hyper" {{args}}
            ;;
        plugin)
            just build-plugin
            cd "$ROOT/editors/jetbrains-plugin" && ./gradlew runIde
            ;;
        *)
            echo "Usage: just run [transpiler|plugin] [args...]"
            exit 1
            ;;
    esac

# Test transpiler or plugin
test target:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"

    case "{{target}}" in
        transpiler)
            cd "$ROOT/rust" && cargo test
            ;;
        plugin)
            echo "Building transpiler (debug)..."
            cd "$ROOT/rust" && cargo build -q
            cd "$ROOT/editors/jetbrains-plugin" && ./gradlew test
            ;;
        *)
            echo "Usage: just test [transpiler|plugin]"
            exit 1
            ;;
    esac

# Update all expected test files from current transpiler output
test-accept *filter:
    cd {{justfile_directory()}}/rust/transpiler && cargo run --example accept_expected -- {{filter}}

# Show diff between expected and actual transpiler output
test-diff *filter:
    #!/usr/bin/env bash
    cd "{{justfile_directory()}}/rust" && cargo test --test expected_tests 2>&1 | head -100

# Run expected tests only (faster than full test suite)
test-expected:
    cd {{justfile_directory()}}/rust && cargo test --test expected_tests

# Bundle transpiler binary into plugin resources (internal helper)
_bundle:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"

    # Detect platform
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
    echo "✓ Bundled: $DEST"

# Generate Python from .hyper files
generate *files:
    {{justfile_directory()}}/rust/target/release/hyper generate {{files}}

# Compile .hyper file(s) (build + run, for quick iteration)
compile *files:
    cd {{justfile_directory()}}/rust && RUSTFLAGS="-Awarnings" cargo run -q -- generate {{files}}