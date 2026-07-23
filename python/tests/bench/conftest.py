"""Fixtures for the render/escape benchmarks."""

from __future__ import annotations

from pathlib import Path

import pytest

from hyperhtml import _autohook
from hyperhtml.integrations._discovery import import_hyper

TEMPLATES = Path(__file__).parent / "templates"


@pytest.fixture(scope="session", autouse=True)
def install_hyper_imports():
    _autohook._install_finder()
    for stale in TEMPLATES.glob("*.py"):
        stale.unlink()
    yield


def _load(name: str):
    loaded = import_hyper(TEMPLATES / f"{name}.hyper")
    return loaded if getattr(loaded, "__hyper__", False) else getattr(loaded, name)


@pytest.fixture
def product_page(install_hyper_imports):
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
