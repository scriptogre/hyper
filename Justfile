# Run format, lint, and security checks
check: format lint security

# Format code with ruff
format:
    uvx ruff format hyper

# Check code for lint errors
lint:
    uvx ruff check hyper

# Check code for type errors
type:
    uvx ty check hyper

# Check code for security issues
security:
    uvx bandit -r hyper

# Run all tests
test:
    uv run pytest .

# Run a playground template
play file:
    #!/usr/bin/env bash
    set -e
    PYTHONPATH="{{justfile_directory()}}" uv run python -c "from hyper import load_component; print(load_component('playground/{{file}}')())"

# Transpiler commands
transpiler command="build" file="":
    #!/usr/bin/env bash
    set -e
    case "{{command}}" in
        build)
            cd {{justfile_directory()}}/rust && cargo build --release -p hyper-transpiler
            ;;
        run)
            if [ -z "{{file}}" ]; then
                echo "Usage: just transpiler run <file>"
                exit 1
            fi
            {{justfile_directory()}}/rust/target/release/hyper generate {{file}}
            ;;
        *)
            # If command looks like a file path, treat it as 'run <file>'
            if [ -f "{{command}}" ]; then
                {{justfile_directory()}}/rust/target/release/hyper generate {{command}}
            else
                echo "Unknown command: {{command}}"
                echo "Usage: just transpiler [build|run <file>]"
                exit 1
            fi
            ;;
    esac

# JetBrains plugin commands
jetbrains command="run":
    #!/usr/bin/env bash
    set -e
    case "{{command}}" in
        run)
            {{justfile_directory()}}/editors/jetbrains-plugin/gradlew -p {{justfile_directory()}}/editors/jetbrains-plugin runIde
            ;;
        build)
            just transpiler build
            just jetbrains bundle
            {{justfile_directory()}}/editors/jetbrains-plugin/gradlew -p {{justfile_directory()}}/editors/jetbrains-plugin build
            ;;
        bundle)
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
            SRC="{{justfile_directory()}}/rust/target/release/hyper"
            DEST="{{justfile_directory()}}/editors/jetbrains-plugin/src/main/resources/bin/${BINARY_NAME}"
            mkdir -p "$(dirname "$DEST")"
            cp "$SRC" "$DEST"
            echo "Bundled transpiler: $DEST"
            ;;
        *)
            echo "Unknown command: {{command}}"
            echo "Usage: just jetbrains [build|bundle|run]"
            exit 1
            ;;
    esac
