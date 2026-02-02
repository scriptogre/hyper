from collections.abc import Iterable
from hyper import component


@component
def NamedWithFallback(_content: Iterable[str] | None = None, *, title: str, _footer_content: Iterable[str] | None = None, _header_content: Iterable[str] | None = None, _sidebar_content: Iterable[str] | None = None):
    yield "<div class=\"layout\">"
    yield "<header>"
    if _header_content is not None:
        yield from _header_content
    yield "</header>"
    yield "<nav>"
    if _sidebar_content is not None:
        yield from _sidebar_content
    yield "</nav>"
    yield "<main>"
    if _content is not None:
        yield from _content
    yield "</main>"
    yield "<footer>"
    if _footer_content is not None:
        yield from _footer_content
    yield "</footer>"
    yield "</div>"
