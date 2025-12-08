# Run all tests with coverage report
test:
    #!/usr/bin/env bash
    set -e
    cd hyper/content

    # Run all tests with full dependencies (msgspec, pydantic)
    uv run --isolated --with-editable '.[test,msgspec,pydantic]' pytest tests/ --cov=hyper.content --cov-report=term-missing
