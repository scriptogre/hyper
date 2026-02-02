from collections.abc import Iterable
from hyper import component


@component
def NamedSlots(_content: Iterable[str] | None = None, _sidebar_content: Iterable[str] | None = None):
    yield "<div class=\"layout\">"
    yield "<aside>"
    if _sidebar_content is not None:
        yield from _sidebar_content
    yield "</aside>"
    yield "<main>"
    if _content is not None:
        yield from _content
    yield "</main>"
    yield "</div>"
