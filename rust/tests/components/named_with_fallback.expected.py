from collections.abc import Iterable
from hyper import html


@html
def NamedWithFallback(_content: Iterable[str] | None = None, *, title: str, _footer: Iterable[str] | None = None, _header: Iterable[str] | None = None, _sidebar: Iterable[str] | None = None):

    yield """<div class="layout">"""

    yield """<header>"""

    if _header is not None:
        yield from _header
    else:
        yield """\
<h1>Default Header</h1>
        """

    yield """</header>"""

    yield """<nav>"""

    if _sidebar is not None:
        yield from _sidebar
    else:
        yield """\
<p>Default sidebar content</p>
        """

    yield """</nav>"""

    yield """<main>"""

    if _content is not None:
        yield from _content

    yield """</main>"""

    yield """<footer>"""

    if _footer is not None:
        yield from _footer
    else:
        yield """\
<p>Default footer</p>
        """

    yield """</footer>"""

    yield """</div>"""

