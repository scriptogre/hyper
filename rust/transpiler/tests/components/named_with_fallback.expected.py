from collections.abc import Iterable
from hyper import component


@component
def NamedWithFallback(
    _content: Iterable[str] | None = None,
    _header: Iterable[str] | None = None,
    _sidebar: Iterable[str] | None = None,
    _footer: Iterable[str] | None = None,
    *,
    title: str
):
    yield """\
<div class="layout">
    <header>"""
    if _header is not None:
        yield from _header
    else:
        yield """<h1>Default Header</h1>"""
    yield """\
</header>

    <nav>"""
    if _sidebar is not None:
        yield from _sidebar
    else:
        yield """<p>Default sidebar content</p>"""
    yield """\
</nav>

    <main>"""
    if _content is not None:
        yield from _content
    yield """\
</main>

    <footer>"""
    if _footer is not None:
        yield from _footer
    else:
        yield """<p>Default footer</p>"""
    yield """\
</footer>
</div>"""
