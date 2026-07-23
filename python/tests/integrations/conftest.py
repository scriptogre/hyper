"""Shared fixtures for integration tests."""

from __future__ import annotations

from pathlib import Path

import pytest

from hyperhtml import _autohook

FIXTURES = Path(__file__).parent / "fixtures"
COMPONENTS_DIR = FIXTURES / "components"


@pytest.fixture(scope="session", autouse=True)
def install_hyper_imports():
    """Load fixture components through the `.hyper` import hook."""
    _autohook._install_finder()
    for stale in COMPONENTS_DIR.rglob("*.py"):
        if stale.name != "__init__.py":
            stale.unlink()
    yield COMPONENTS_DIR


@pytest.fixture
def fixtures_dir() -> Path:
    return FIXTURES


@pytest.fixture
def components_dir() -> Path:
    return COMPONENTS_DIR


@pytest.fixture(autouse=True)
def _clear_component_registry():
    """Wipe the Django registry between tests so leakage doesn't mask bugs."""
    try:
        from django.apps import apps  # type: ignore

        config = apps.get_app_config("hyper")
    except Exception:
        yield
        return
    config.components = {}
    yield
    config.components = {}
