# Run all tests with coverage report
test:
    uv run pytest .

# Run a playground page or component
play file:
    #!/usr/bin/env bash
    set -e
    PYTHONPATH="{{justfile_directory()}}" uv run python -c "from hyper import load_component; print(load_component('playground/{{file}}')())"
