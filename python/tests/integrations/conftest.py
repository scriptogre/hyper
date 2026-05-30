"""Shared fixtures for integration tests.

Compiles the .hyper fixture components once per test session so each test
file gets fresh .py siblings.
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

FIXTURES = Path(__file__).parent / "fixtures"
COMPONENTS_DIR = FIXTURES / "components"
BINARY = Path(__file__).resolve().parents[3] / "rust" / "target" / "release" / "hyper"


@pytest.fixture(scope="session", autouse=True)
def compile_fixture_components():
    """Compile every .hyper file under fixtures/ before the integration tests run.

    Fails the session loudly when the release binary is missing — tests rely on
    real compiled output, not hand-written stubs.
    """
    if not BINARY.exists():
        pytest.skip(
            f"Hyper release binary not found at {BINARY}. "
            "Run 'just build' first.",
            allow_module_level=True,
        )

    # Wipe stale .py siblings so we never run against orphan compiled artifacts.
    for stale in COMPONENTS_DIR.rglob("*.py"):
        if stale.name == "__init__.py":
            continue
        stale.unlink()

    result = subprocess.run(
        [str(BINARY), "generate", str(COMPONENTS_DIR)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        pytest.fail(
            f"Failed to compile fixture components:\n"
            f"stdout: {result.stdout}\nstderr: {result.stderr}"
        )

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
        from hyper.integrations.django import _registry  # type: ignore
    except Exception:
        yield
        return
    _registry.set_components({})
    yield
    _registry.set_components({})
