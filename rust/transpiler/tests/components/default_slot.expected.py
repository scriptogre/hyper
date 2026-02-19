from collections.abc import Iterable
from hyper import html, replace_markers


@html
def DefaultSlot(_content: Iterable[str] | None = None, *, title: str):

    yield "<div class=\"card\">"
    yield replace_markers(f"""\
<h2>‹ESCAPE:{title}›</h2>
    """)
    if _content is not None:
        yield from _content

    yield "</div>"

