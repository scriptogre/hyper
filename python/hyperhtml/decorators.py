"""Lazy Hyper component instances."""

from __future__ import annotations

import functools
import inspect
from collections.abc import AsyncIterator, Callable, Iterable, Iterator
from typing import Any, overload

__all__ = ["Component", "HtmlResult", "component", "subcomponent"]


class HtmlResult(str):
    """Rendered component output that HTML integrations treat as safe."""

    __slots__ = ()

    def __html__(self) -> str:
        return self


class Component:
    """A component factory or a lazy component instance with bound props."""

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

        self._render = render
        self._is_async = inspect.isasyncgenfunction(render)
        self._props: dict[str, Any] | None = None
        self._subcomponent_names = frozenset()

        functools.update_wrapper(self, render, updated=())
        self.__signature__ = signature
        self.__hyper__ = True
        self.do_not_call_in_templates = True

        names: set[str] = set()
        for child in subcomponents:
            if not isinstance(child, Component):
                raise TypeError("subcomponents must be Component objects")
            name = child.__name__
            if name in names:
                raise TypeError(f"duplicate subcomponent {name!r}")
            if hasattr(type(self), name) or name in self.__dict__:
                raise TypeError(f"{name!r} is reserved by Component")
            names.add(name)
            object.__setattr__(self, name, child)

        self._subcomponent_names = frozenset(names)

    def __call__(self, **props: Any) -> Component:
        if self._props is not None:
            raise TypeError(f"{self.__name__} is already bound")
        self.__signature__.bind(**props)
        instance = object.__new__(type(self))
        instance.__dict__ = self.__dict__.copy()
        instance._props = props
        return instance

    @overload
    def render(self, *, stream: bool = False) -> HtmlResult | Any: ...

    @overload
    def render(self, *, stream: bool = True) -> Iterator[str] | AsyncIterator[str]: ...

    def render(
        self, *, stream: bool = False
    ) -> HtmlResult | Iterator[str] | AsyncIterator[str] | Any:
        if self._props is None:
            raise TypeError(f"Bind {self.__name__} props before rendering")
        if stream:
            return self._render(**self._props)
        if self._is_async:
            return self._buffer_async()
        return HtmlResult("".join(self._render(**self._props)))

    async def _buffer_async(self) -> HtmlResult:
        chunks = [chunk async for chunk in self._render(**self._props)]
        return HtmlResult("".join(chunks))

    def __str__(self) -> str:
        if self._props is None:
            return repr(self)
        if self._is_async:
            raise TypeError(
                f"{self.__name__} is async; use await {self.__name__}(...).render()"
            )
        return self.render()

    def __repr__(self) -> str:
        if self._props is None:
            return f"<Component {self.__name__}>"
        values = ", ".join(f"{name}={value!r}" for name, value in self._props.items())
        return f"<{self.__name__}({values})>"

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
    if render is None:
        return lambda function: Component(function, subcomponents=subcomponents)
    return Component(render, subcomponents=subcomponents)


def subcomponent(definition: Any) -> Any:
    """Mark a nested component definition for export by the Hyper compiler."""
    return definition
