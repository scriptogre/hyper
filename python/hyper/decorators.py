"""Decorators for Hyper templates.

@html wraps a generator function. Calling the wrapped function eagerly
renders the component to an HtmlResult — a real ``str`` subclass — so the
component works natively anywhere Python frameworks accept strings:

    @html
    def Sidebar(*, user):
        yield f"<aside>{user}</aside>"

    Sidebar(user="Ada")              # HtmlResult('<aside>Ada</aside>')
    isinstance(Sidebar(user="Ada"), str)   # True

    # FastAPI, Flask, Django — all native:
    return Sidebar(user="Ada")

    # Streaming (chunk-by-chunk, no materialization):
    for chunk in Sidebar.stream(user="Ada"):
        ...

Async components return a coroutine that resolves to an HtmlResult:

    @html
    async def Page(*, title):
        yield f"<title>{title}</title>"

    await Page(title="x")            # HtmlResult, awaited
    async for chunk in Page.stream(title="x"):   # async streaming
        ...
"""

from __future__ import annotations

import functools
import inspect

__all__ = ["html", "HtmlResult"]


class HtmlResult(str):
    """Rendered HTML output from a Hyper component.

    A genuine ``str`` subclass: anywhere a framework, template, or library
    expects a string, an HtmlResult is one. The ``__html__`` method opts the
    value out of further escaping under the MarkupSafe protocol (Jinja,
    MarkupSafe consumers).
    """

    __slots__ = ()

    def __html__(self) -> str:
        # MarkupSafe protocol: signals "already HTML, don't escape me again."
        return self


def html(fn):
    """Decorator that wraps a generator function as a Hyper component.

    The wrapped callable returns ``HtmlResult`` (eagerly rendered). Use the
    attached ``.stream()`` method to get the raw chunk iterator instead.
    """
    sig = inspect.signature(fn)

    if inspect.isasyncgenfunction(fn):

        @functools.wraps(fn)
        async def async_wrapper(*args, **kwargs):
            sig.bind(*args, **kwargs)
            chunks: list[str] = []
            async for chunk in fn(*args, **kwargs):
                chunks.append(chunk)
            return HtmlResult("".join(chunks))

        def async_stream(*args, **kwargs):
            sig.bind(*args, **kwargs)
            return fn(*args, **kwargs)  # async generator

        async_wrapper.stream = async_stream
        # Discovery marker for hyper.integrations.* component discovery.
        async_wrapper.__hyper__ = True
        # Django: prevents the template engine from auto-calling the wrapped
        # function during variable resolution (which would invoke it with no
        # args). The {% hyper %} tag does the call with kwargs.
        async_wrapper.do_not_call_in_templates = True
        return async_wrapper

    @functools.wraps(fn)
    def wrapper(*args, **kwargs):
        sig.bind(*args, **kwargs)
        return HtmlResult("".join(fn(*args, **kwargs)))

    def stream(*args, **kwargs):
        sig.bind(*args, **kwargs)
        return fn(*args, **kwargs)  # raw sync generator of chunks

    wrapper.stream = stream
    wrapper.__hyper__ = True
    wrapper.do_not_call_in_templates = True
    return wrapper
