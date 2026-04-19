from collections.abc import Iterable
from hyper import html, escape


@html
def DefaultSlot(_content: Iterable[str] | None = None, *, title: str):

    yield """<div class="card">"""
    yield f"""\
<h2>{escape(title)}</h2>
    """
    if _content is not None:
        yield from _content

    yield """</div>"""

