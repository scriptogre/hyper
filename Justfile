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
