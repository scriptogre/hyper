from collections.abc import Iterable
from hyper import html


@html
def NamedSlots(_content: Iterable[str] | None = None, _sidebar: Iterable[str] | None = None):
    yield """<div class="layout">"""
    yield """<aside>"""
    if _sidebar is not None:
        yield from _sidebar
    yield """</aside>"""
    yield """<main>"""
    if _content is not None:
        yield from _content
    yield """</main>"""
    yield """</div>"""
