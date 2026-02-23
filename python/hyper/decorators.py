"""Decorators for Hyper templates.

The @html decorator wraps generator functions to support both modes:
- str(Component(...)) for buffered output
- iter(Component(...)) / yield from Component(...) for streaming
"""

import asyncio
import functools
import inspect

__all__ = ["html"]


class HtmlResult:
    """Iterable, str()-able result from an @html component."""
    __slots__ = ("_fn", "_args", "_kwargs")

    def __init__(self, fn, args, kwargs):
        self._fn = fn
        self._args = args
        self._kwargs = kwargs

    def __iter__(self):
        return iter(self._fn(*self._args, **self._kwargs))

    def __str__(self):
        return "".join(self._fn(*self._args, **self._kwargs))

    def __repr__(self):
        return f"HtmlResult({self._fn.__name__})"


class AsyncHtmlResult:
    """Async iterable, renderable result from an async @html component."""
    __slots__ = ("_fn", "_args", "_kwargs")

    def __init__(self, fn, args, kwargs):
        self._fn = fn
        self._args = args
        self._kwargs = kwargs

    def __aiter__(self):
        return self._fn(*self._args, **self._kwargs)

    async def render(self):
        """Render component to string (async)."""
        chunks = []
        async for chunk in self._fn(*self._args, **self._kwargs):
            chunks.append(chunk)
        return "".join(chunks)

    def __str__(self):
        try:
            asyncio.get_running_loop()
            raise RuntimeError(
                "Use 'await component.render()' in async context, "
                "or use yield mode with 'async for'"
            )
        except RuntimeError as e:
            if "no running event loop" in str(e).lower():
                return asyncio.run(self.render())
            raise

    def __repr__(self):
        return f"AsyncHtmlResult({self._fn.__name__})"


def html(fn):
    """Decorator that wraps a generator function for HTML template output.

    The wrapped function returns an HtmlResult (or AsyncHtmlResult) that supports:
    - str(result) for buffered output
    - iter(result) / yield from result for streaming
    - async for chunk in result for async streaming
    """
    sig = inspect.signature(fn)

    if inspect.isasyncgenfunction(fn):
        @functools.wraps(fn)
        def async_wrapper(*args, **kwargs):
            sig.bind(*args, **kwargs)  # validate arguments eagerly
            return AsyncHtmlResult(fn, args, kwargs)
        return async_wrapper

    @functools.wraps(fn)
    def wrapper(*args, **kwargs):
        sig.bind(*args, **kwargs)  # validate arguments eagerly
        return HtmlResult(fn, args, kwargs)
    return wrapper
