"""Fixtures for the render/escape benchmarks.

Compiles the .hyper templates once per session with the release binary, then
imports the generated component so benchmarks run against real output.
"""

from __future__ import annotations

import importlib.util
import subprocess
from pathlib import Path

import pytest

TEMPLATES = Path(__file__).parent / "templates"
BINARY = Path(__file__).resolve().parents[3] / "rust" / "target" / "release" / "hyper"


@pytest.fixture(scope="session", autouse=True)
def compile_templates():
    if not BINARY.exists():
        pytest.skip(
            f"Hyper release binary not found at {BINARY}. Run 'just build' first.",
            allow_module_level=True,
        )

    for stale in TEMPLATES.glob("*.py"):
        stale.unlink()

    result = subprocess.run(
        [str(BINARY), "generate", str(TEMPLATES)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        pytest.fail(f"Failed to compile templates:\nstderr: {result.stderr}")

    yield


def _load(name: str):
    spec = importlib.util.spec_from_file_location(name, TEMPLATES / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return getattr(module, name)


@pytest.fixture
def product_page(compile_templates):
    return _load("ProductPage")


def make_products(n: int) -> list[dict]:
    """Sample data with a realistic mix of plain and escapable strings."""
    return [
        {
            "id": i,
            "name": f"Widget {i}" if i % 3 else f"Bolts & Nuts <{i}>",
            "description": "A sturdy, well-made item for everyday use.",
            "price": f"{i * 1.5:.2f}",
        }
        for i in range(n)
    ]
