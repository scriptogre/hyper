from collections.abc import Iterable
from hyper import component, escape


@component
def DefaultSlot(_content: Iterable[str] | None = None, *, title: str):
    yield "<div class=\"card\">"
    yield "<h2>"
    yield escape(title)
    yield "</h2>"
    if _content is not None:
        yield from _content
    yield "</div>"
