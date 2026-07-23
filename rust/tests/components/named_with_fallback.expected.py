from collections.abc import Iterable
from hyperhtml import component


@component
def NamedWithFallback(
        *,
        title: str,
        content: Iterable[str] | None = None,
        footer: Iterable[str] | None = None,
        header: Iterable[str] | None = None,
        sidebar: Iterable[str] | None = None,
):
    yield """<div class="layout">"""
    yield """<header>"""
    # <{...header}>
    if header is not None:
        yield from header
    else:
        yield """<h1>Default Header</h1>"""
    # </{...header}>
    yield """</header>"""

    yield """<nav>"""
    # <{...sidebar}>
    if sidebar is not None:
        yield from sidebar
    else:
        yield """<p>Default sidebar content</p>"""
    # </{...sidebar}>
    yield """</nav>"""

    yield """<main>"""
    # <{...}>
    if content is not None:
        yield from content
    # </{...}>
    yield """</main>"""

    yield """<footer>"""
    # <{...footer}>
    if footer is not None:
        yield from footer
    else:
        yield """<p>Default footer</p>"""
    # </{...footer}>
    yield """</footer>"""
    yield """</div>"""
