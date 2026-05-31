from collections.abc import Iterable
from hyper import html


@html
def NamedWithFallback(
        _default_slot: Iterable[str] | None = None,
        *,
        title: str,
        _footer_slot: Iterable[str] | None = None,
        _header_slot: Iterable[str] | None = None,
        _sidebar_slot: Iterable[str] | None = None,
):
    yield """<div class="layout">"""
    yield """<header>"""
    # <{...header}>
    if _header_slot is not None:
        yield from _header_slot
    else:
        yield """<h1>Default Header</h1>"""
    # </{...header}>
    yield """</header>"""

    yield """<nav>"""
    # <{...sidebar}>
    if _sidebar_slot is not None:
        yield from _sidebar_slot
    else:
        yield """<p>Default sidebar content</p>"""
    # </{...sidebar}>
    yield """</nav>"""

    yield """<main>"""
    # <{...}>
    if _default_slot is not None:
        yield from _default_slot
    # </{...}>
    yield """</main>"""

    yield """<footer>"""
    # <{...footer}>
    if _footer_slot is not None:
        yield from _footer_slot
    else:
        yield """<p>Default footer</p>"""
    # </{...footer}>
    yield """</footer>"""
    yield """</div>"""
