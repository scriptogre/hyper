"""Internal helpers for discovering Hyper components on the filesystem or via package walking.

Shared by the Jinja2 and Django integrations. Not part of the public API.
"""

from __future__ import annotations

import importlib
import importlib.util
import pkgutil
import sys
import warnings
from collections.abc import Iterator
from pathlib import Path


def file_to_module_name(py_path: Path) -> tuple[str, Path] | None:
    """Resolve a compiled .py file to (dotted_name, sys_path_root) by walking up __init__.py.

    Returns None when the file isn't inside a proper Python package chain.
    """
    if not py_path.exists():
        return None
    parts = [py_path.stem]
    parent = py_path.parent
    while (parent / "__init__.py").exists():
        parts.insert(0, parent.name)
        parent = parent.parent
    if len(parts) == 1:
        return None
    return ".".join(parts), parent


def import_compiled(py_path: Path):
    """Import a compiled .py file. Prefers normal import resolution so sibling imports work;
    falls back to file-based loading for orphan files (no __init__.py chain).
    """
    resolved = file_to_module_name(py_path)
    if resolved is not None:
        dotted, root = resolved
        root_str = str(root)
        if root_str not in sys.path:
            sys.path.insert(0, root_str)
        return importlib.import_module(dotted)

    name = f"_hyper_orphan_{py_path.stem}_{abs(hash(str(py_path.resolve())))}"
    spec = importlib.util.spec_from_file_location(name, py_path)
    if spec is None or spec.loader is None:
        return None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def discover_in_path(path: Path) -> Iterator[tuple[str, object]]:
    """Walk a directory for .hyper files; yield (name, component) for each compiled sibling."""
    if not path.exists():
        return
    for hyper_file in path.rglob("*.hyper"):
        py_file = hyper_file.with_suffix(".py")
        if not py_file.exists():
            warnings.warn(
                f"Hyper: found {hyper_file} but no compiled sibling "
                f"'{py_file.name}'. Run 'hyper generate' to compile.",
                stacklevel=3,
            )
            continue
        try:
            module = import_compiled(py_file)
        except Exception as e:
            warnings.warn(
                f"Hyper: failed to import compiled component {py_file}: {e!r}",
                stacklevel=3,
            )
            continue
        if module is None:
            continue
        for name, attr in vars(module).items():
            if getattr(attr, "__hyper__", False):
                yield name, attr


def discover_in_package(package) -> Iterator[tuple[str, object]]:
    """Walk a Python package; yield (name, component) for every __hyper__-marked export."""
    for _, modname, _ in pkgutil.walk_packages(
        package.__path__, prefix=package.__name__ + "."
    ):
        try:
            module = importlib.import_module(modname)
        except Exception as e:
            warnings.warn(
                f"Hyper: failed to import {modname}: {e!r}",
                stacklevel=3,
            )
            continue
        for name, attr in vars(module).items():
            if getattr(attr, "__hyper__", False):
                yield name, attr
