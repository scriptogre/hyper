"""Decorators for Hyper templates.

The @component decorator makes generator functions work in both modes:
- Yield mode: `yield from Component(_content)` or `async for chunk in Component(_content)`
- Buffer mode: `with Component() as _c: _c.append(...)`

Supports both sync and async generator functions automatically.

Components with slots (accepts _content as first parameter):
    @component
    def Card(_content=None, *, title=""):
        yield f'<div class="card"><h1>{title}</h1>'
        if _content:
            yield from _content
        yield '</div>'

    # Yield mode
    html = "".join(Card(title="Hello"))

    # Buffer mode
    with Card(title="Hello") as card:
        card.append("<p>Content</p>")
    html = str(card)

Components without slots (no _content parameter):
    @component
    def Badge(*, text="", color="blue"):
        yield f'<span class="badge" style="color: {color}">{text}</span>'

    # Yield mode
    html = "".join(Badge(text="New", color="red"))

    # As string
    html = str(Badge(text="New"))

Async components:
    @component
    async def Card(_content=None, *, title=""):
        data = await fetch_data()
        yield f'<div class="card"><h1>{title} - {data}</h1>'
        if _content:
            async for chunk in _content:
                yield chunk
        yield '</div>'

    # Yield mode
    chunks = [chunk async for chunk in Card(title="Hello")]
    html = "".join(chunks)

    # Buffer mode
    async with Card(title="Hello") as card:
        card.append("<p>Content</p>")
    html = await card.render()
"""

import asyncio
import inspect

__all__ = ["component"]


def component(fn):
    """Decorator that wraps a generator function to support both yield and buffer modes.

    Automatically detects sync vs async generator functions and returns the appropriate wrapper.

    Args:
        fn: A generator function (sync or async) that yields HTML chunks.

    Returns:
        A wrapper class that can be used as an iterator, async iterator, or context manager.
    """
    if inspect.isasyncgenfunction(fn):
        return _make_async_wrapper(fn)
    else:
        return _make_sync_wrapper(fn)


def _make_sync_wrapper(fn):
    """Create wrapper for synchronous generator functions."""
    # Check if the function accepts _content as its first parameter
    sig = inspect.signature(fn)
    params = list(sig.parameters.keys())
    has_content_param = params and params[0] == "_content"

    class SyncComponentWrapper:
        __slots__ = ("_content", "_props", "_collected")

        def __init__(self, _content=None, **props):
            self._content = _content
            self._props = props
            self._collected = []

        # Buffer mode - context manager
        def __enter__(self):
            return self

        def __exit__(self, *args):
            pass

        def append(self, content):
            """Append content (buffer mode)."""
            self._collected.append(str(content))

        def _call_fn(self):
            """Call the wrapped function with appropriate arguments."""
            if has_content_param:
                content = self._get_content()
                return fn(content, **self._props)
            else:
                return fn(**self._props)

        # Yield mode - make it iterable
        def __iter__(self):
            return iter(self._call_fn())

        # Buffer mode - render to string
        def __str__(self):
            return "".join(self._call_fn())

        def _get_content(self):
            if self._collected:
                def gen():
                    for item in self._collected:
                        yield item
                return gen()
            else:
                return self._content

    SyncComponentWrapper.__name__ = fn.__name__
    SyncComponentWrapper.__qualname__ = fn.__qualname__
    SyncComponentWrapper.__doc__ = fn.__doc__
    SyncComponentWrapper.__wrapped__ = fn

    return SyncComponentWrapper


def _make_async_wrapper(fn):
    """Create wrapper for asynchronous generator functions."""
    # Check if the function accepts _content as its first parameter
    sig = inspect.signature(fn)
    params = list(sig.parameters.keys())
    has_content_param = params and params[0] == "_content"

    class AsyncComponentWrapper:
        __slots__ = ("_content", "_props", "_collected")

        def __init__(self, _content=None, **props):
            self._content = _content
            self._props = props
            self._collected = []

        # Buffer mode - async context manager
        async def __aenter__(self):
            return self

        async def __aexit__(self, *args):
            pass

        def append(self, content):
            """Append content (buffer mode)."""
            self._collected.append(str(content))

        def _call_fn(self):
            """Call the wrapped function with appropriate arguments."""
            if has_content_param:
                content = self._get_content()
                return fn(content, **self._props)
            else:
                return fn(**self._props)

        # Yield mode - make it async iterable
        def __aiter__(self):
            return self._call_fn()

        # Buffer mode - render to string (async)
        async def render(self):
            """Render component to string (async)."""
            chunks = []
            async for chunk in self._call_fn():
                chunks.append(chunk)
            return "".join(chunks)

        # Sync __str__ for convenience (runs event loop if needed)
        def __str__(self):
            try:
                asyncio.get_running_loop()
                # Already in async context - can't use asyncio.run
                raise RuntimeError(
                    "Use 'await component.render()' in async context, "
                    "or use yield mode with 'async for'"
                )
            except RuntimeError as e:
                if "no running event loop" in str(e).lower():
                    # No running loop - safe to use asyncio.run
                    return asyncio.run(self.render())
                raise

        def _get_content(self):
            if self._collected:
                async def gen():
                    for item in self._collected:
                        yield item
                return gen()
            else:
                return self._content

    AsyncComponentWrapper.__name__ = fn.__name__
    AsyncComponentWrapper.__qualname__ = fn.__qualname__
    AsyncComponentWrapper.__doc__ = fn.__doc__
    AsyncComponentWrapper.__wrapped__ = fn

    return AsyncComponentWrapper
