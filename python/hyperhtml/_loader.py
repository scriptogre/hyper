"""Import `.hyper` files as Python modules and package attributes."""

from __future__ import annotations

import importlib.abc
import importlib.machinery
import sys
import threading
from pathlib import Path
from types import ModuleType
from typing import Iterable


class HyperFinder(importlib.abc.MetaPathFinder):
    """Find packages and modules backed by `.hyper` files."""

    def find_spec(self, fullname: str, path=None, target=None):
        name = fullname.rpartition(".")[2]

        for base in _search_paths(path):
            # Normal Python modules keep their standard import precedence.
            if (base / f"{name}.py").is_file():
                return None

            module_path = base / f"{name}.hyper"
            if module_path.is_file():
                code, component_name = _compile_file(module_path)
                if component_name is not None:
                    return None
                return importlib.machinery.ModuleSpec(
                    fullname,
                    HyperModuleLoader(module_path, code),
                    origin=str(module_path),
                )

            package_path = base / name
            if package_path.is_dir() and _contains_hyper_file(package_path):
                loader = HyperPackageLoader(package_path)
                spec = importlib.machinery.ModuleSpec(
                    fullname,
                    loader,
                    origin=str(package_path / "__init__.py"),
                    is_package=True,
                )
                spec.submodule_search_locations = [str(package_path)]
                return spec

        return None


class HyperPackageLoader(importlib.abc.Loader):
    """Load a directory package and expose `{Name}.hyper` as `Name`."""

    def __init__(self, path: Path):
        self.path = path
        self._lock = threading.RLock()

    def create_module(self, spec):
        return None

    def exec_module(self, module: ModuleType) -> None:
        module.__file__ = str(self.path / "__init__.py")
        module.__path__ = [str(self.path)]

        init_path = self.path / "__init__.py"
        if init_path.is_file():
            source = init_path.read_text()
            exec(compile(source, str(init_path), "exec"), module.__dict__)

        user_getattr = module.__dict__.get("__getattr__")

        def __getattr__(name: str):
            if user_getattr is not None:
                try:
                    return user_getattr(name)
                except AttributeError:
                    pass

            if name.startswith("__"):
                raise AttributeError(name)

            python_path = self.path / f"{name}.py"
            if python_path.is_file():
                return importlib.import_module(f"{module.__name__}.{name}")

            hyper_path = self.path / f"{name}.hyper"
            if not hyper_path.is_file():
                raise AttributeError(name)

            with self._lock:
                if name in module.__dict__:
                    return module.__dict__[name]

                code, component_name = _compile_file(hyper_path)
                if component_name is None:
                    return importlib.import_module(f"{module.__name__}.{name}")

                namespace = {
                    "__name__": f"{module.__name__}.{name}",
                    "__package__": module.__name__,
                    "__file__": str(hyper_path),
                    "__loader__": HyperModuleLoader(hyper_path),
                }
                exec(code, namespace)

                try:
                    component = namespace[component_name]
                except KeyError as exc:
                    raise ImportError(
                        f"{hyper_path} did not define its component {component_name!r}"
                    ) from exc

                module.__dict__[name] = component
                return component

        module.__getattr__ = __getattr__


class HyperModuleLoader(importlib.abc.Loader):
    """Load one `.hyper` file as a normal Python module."""

    def __init__(self, path: Path, code: str | None = None):
        self.path = path
        self.code = code

    def create_module(self, spec):
        return None

    def exec_module(self, module: ModuleType) -> None:
        module.__file__ = str(self.path)
        module.__package__ = module.__name__.rpartition(".")[0]
        code = self.code if self.code is not None else _compile_file(self.path)[0]
        exec(code, module.__dict__)


def _search_paths(path) -> Iterable[Path]:
    entries = sys.path if path is None else path
    for entry in entries:
        if not entry:
            entry = "."
        try:
            yield Path(entry)
        except TypeError:
            continue


def _contains_hyper_file(path: Path) -> bool:
    try:
        next(path.rglob("*.hyper"))
        return True
    except (StopIteration, OSError):
        return False


def _compile_file(path: Path) -> tuple[str, str | None]:
    try:
        from hyperhtml import _native
    except Exception as exc:  # pragma: no cover - depends on wheel build health
        raise ImportError(
            "Hyper's native compiler extension is not available. "
            "Reinstall the hyperhtml package for your platform."
        ) from exc

    try:
        source = path.read_text()
        return _native.transpile_file(source, str(path))
    except Exception as exc:
        raise ImportError(f"Failed to compile {path}: {exc}") from exc
