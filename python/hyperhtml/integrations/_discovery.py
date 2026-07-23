"""Internal helpers for discovering Hyper components on the filesystem or via package walking.

Shared by the Jinja2 and Django integrations. Not part of the public API.
"""

from __future__ import annotations

import importlib
import importlib.util
import sys
import warnings
from collections.abc import Iterator
from pathlib import Path

from hyperhtml._autohook import _install_finder
from hyperhtml._loader import HyperModuleLoader


def file_to_module_name(path: Path) -> tuple[str, Path] | None:
    """Resolve a source file to (dotted_name, sys_path_root) by walking up __init__.py.

    Returns None when the file isn't inside a proper Python package chain.
    """
    if not path.exists():
        return None
    parts = [path.stem]
    parent = path.parent
    while (parent / "__init__.py").exists():
        parts.insert(0, parent.name)
        parent = parent.parent
    if len(parts) == 1:
        return None
    return ".".join(parts), parent


def import_hyper(hyper_path: Path):
    """Import a `.hyper` file through the in-process compiler."""
    _install_finder()
    resolved = file_to_module_name(hyper_path)
    if resolved is not None:
        dotted, root = resolved
        root_str = str(root)
        if root_str not in sys.path:
            sys.path.insert(0, root_str)

        package_name, _, name = dotted.rpartition(".")
        package = importlib.import_module(package_name)
        try:
            return getattr(package, name)
        except AttributeError:
            # Library files retain normal submodule imports.
            return importlib.import_module(dotted)

    name = f"_hyper_orphan_{hyper_path.stem}_{abs(hash(str(hyper_path.resolve())))}"
    spec = importlib.util.spec_from_loader(
        name,
        HyperModuleLoader(hyper_path),
        origin=str(hyper_path),
    )
    if spec is None or spec.loader is None:
        return None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def discover_in_path(path: Path) -> Iterator[tuple[str, object]]:
    """Walk a directory for `.hyper` files; yield (name, component) for each one."""
    if not path.exists():
        return
    for hyper_file in path.rglob("*.hyper"):
        try:
            loaded = import_hyper(hyper_file)
        except Exception as e:
            warnings.warn(
                f"Hyper: failed to import component {hyper_file}: {e!r}",
                stacklevel=3,
            )
            continue
        if loaded is None:
            continue
        if getattr(loaded, "__hyper__", False):
            yield loaded.__name__, loaded
            continue
        for name, attr in vars(loaded).items():
            if getattr(attr, "__hyper__", False):
                yield name, attr


def discover_in_package(package) -> Iterator[tuple[str, object]]:
    """Walk a Python package; yield (name, component) for every `.hyper` file."""
    for package_path in package.__path__:
        yield from discover_in_path(Path(package_path))
