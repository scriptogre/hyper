"""
@html turns a generator into a component: call it and get HTML back.

    @html
    def Sidebar(*, user):
        yield f"<aside>{user}</aside>"

    Sidebar(user="Ada")         # "<aside>Ada</aside>", a real str
    Sidebar.stream(user="Ada")  # iterator of chunks, nothing materialized

Async components work the same, awaited:

    await Page(title="x")
    async for chunk in Page.stream(title="x"):
        ...
"""

from __future__ import annotations

import functools
import inspect

__all__ = ["html", "HtmlResult"]


class HtmlResult(str):
    """
    A component's output. A real ``str``, so it works anywhere a string does.
    ``__html__`` tells Jinja/MarkupSafe it's already HTML, not to escape again.
    """

    __slots__ = ()

    def __html__(self) -> str:
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
