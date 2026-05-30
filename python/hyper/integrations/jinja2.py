"""Jinja2 integration for Hyper components.

Drop the extension into a Jinja env and Hyper components living in the env's
loader paths become available as Jinja globals automatically:

    env = Environment(loader=FileSystemLoader("templates"))
    env.add_extension("hyper.integrations.jinja2.HyperExtension")

    # templates/Greeting.hyper compiled to templates/Greeting.py
    # In any Jinja template:    {{ Greeting(name="Ada") }}

For components that don't live next to Jinja templates, use the escape hatch
the extension attaches to the env:

    env.register_hyper_components(myapp.components)
    env.register_hyper_components(SomeComponent, [Other, Another])
"""

from __future__ import annotations

import warnings
from collections.abc import Iterable, Iterator
from pathlib import Path

from jinja2 import BaseLoader, ChoiceLoader, FileSystemLoader
from jinja2.ext import Extension

from hyper.integrations._discovery import discover_in_package, discover_in_path

__all__ = ["HyperExtension"]


def _loader_paths(loader: BaseLoader | None) -> Iterator[Path]:
    """Yield filesystem paths for a Jinja loader, walking ChoiceLoader recursively.

    Returns nothing for loaders we can't introspect (PackageLoader, DictLoader,
    FunctionLoader, etc.). Those projects use the explicit
    `env.register_hyper_components(...)` escape hatch instead.
    """
    if loader is None:
        return
    if isinstance(loader, FileSystemLoader):
        for p in loader.searchpath:
            yield Path(p)
    elif isinstance(loader, ChoiceLoader):
        for child in loader.loaders:
            yield from _loader_paths(child)


class HyperExtension(Extension):
    """Jinja2 extension: discovers and registers Hyper components.

    At init time, walks the environment's loader for `.hyper` files. For each,
    imports the sibling `.py` and registers every `@html`-decorated callable
    (those carry `__hyper__ = True`) into `env.globals` keyed by function name.

    Also attaches `env.register_hyper_components(*sources)` for components that
    don't live next to Jinja templates. Accepts packages, single callables, or
    iterables.
    """

    def __init__(self, environment):
        super().__init__(environment)
        environment.extend(register_hyper_components=self._register)
        self._auto_discover(environment)

    # --- internals -----------------------------------------------------------

    def _auto_discover(self, environment) -> None:
        paths = list(_loader_paths(environment.loader))
        if not paths:
            warnings.warn(
                "Hyper: Jinja loader has no introspectable filesystem paths. "
                "Components won't auto-register. Call "
                "env.register_hyper_components(...) explicitly.",
                stacklevel=3,
            )
            return
        seen: set[str] = set()
        for path in paths:
            for name, component in discover_in_path(path):
                if name in seen:
                    continue
                seen.add(name)
                environment.globals[name] = component

    def _register(self, *targets) -> None:
        env = self.environment
        for t in targets:
            if t is None:
                continue
            if hasattr(t, "__path__"):           # python package
                for name, component in discover_in_package(t):
                    env.globals[name] = component
            elif callable(t):                    # single component
                env.globals[t.__name__] = t
            elif isinstance(t, Iterable):        # iterable of any of the above
                self._register(*t)
            else:
                raise TypeError(
                    f"register_hyper_components: unsupported argument {t!r}. "
                    "Expected a package, a callable, or an iterable thereof."
                )
