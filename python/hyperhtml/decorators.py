"""Callable and streamable Hyper components."""

from __future__ import annotations

import functools
import inspect
from collections.abc import Callable, Iterable
from typing import Any

__all__ = ["Component", "HtmlResult", "component"]


class HtmlResult(str):
    """Rendered component output that HTML integrations treat as safe."""

    __slots__ = ()

    def __html__(self) -> str:
        return self


class Component:
    """A callable component with direct access to its render stream."""

    def __init__(
        self,
        render: Callable[..., Any],
        *,
        subcomponents: Iterable[Component] = (),
    ) -> None:
        signature = inspect.signature(render)
        for parameter in signature.parameters.values():
            if parameter.kind not in {
                inspect.Parameter.KEYWORD_ONLY,
                inspect.Parameter.VAR_KEYWORD,
            }:
                raise TypeError(
                    f"{render.__name__} component parameters must be keyword-only"
                )

        self.stream = render
        self._is_async = inspect.isasyncgenfunction(render)
        self._subcomponent_names = frozenset()

        functools.update_wrapper(self, render, updated=())
        self.__signature__ = signature
        self.__hyper__ = True
        self.do_not_call_in_templates = True

        children = tuple(subcomponents)
        names: set[str] = set()

        for child in children:
            if not isinstance(child, Component):
                raise TypeError("subcomponents must be Component objects")

            name = child.__name__
            if name in names:
                raise TypeError(f"duplicate subcomponent {name!r}")
            if hasattr(type(self), name) or name in self.__dict__:
                raise TypeError(f"{name!r} is reserved by Component")

            names.add(name)

        for child in children:
            object.__setattr__(self, child.__name__, child)

        self._subcomponent_names = frozenset(names)

    def __call__(self, **props: Any) -> HtmlResult | Any:
        if self._is_async:
            return self._buffer_async(**props)
        return HtmlResult("".join(self.stream(**props)))

    async def _buffer_async(self, **props: Any) -> HtmlResult:
        chunks = [chunk async for chunk in self.stream(**props)]
        return HtmlResult("".join(chunks))

    def __setattr__(self, name: str, value: Any) -> None:
        if name in self.__dict__.get("_subcomponent_names", ()):
            raise AttributeError(f"{self.__name__}.{name} is read-only")
        object.__setattr__(self, name, value)

    def __getattr__(self, name: str) -> Component:
        component_name = self.__dict__.get("__name__", "Component")
        raise AttributeError(f"{component_name} has no component {name!r}") from None


def component(
    render: Callable[..., Any] | None = None,
    *,
    subcomponents: Iterable[Component] = (),
) -> Component | Callable[[Callable[..., Any]], Component]:
    """Create a component from a generator function."""

    def decorate(render: Callable[..., Any]) -> Component:
        return Component(render, subcomponents=subcomponents)

    if render is None:
        return decorate
    return decorate(render)
