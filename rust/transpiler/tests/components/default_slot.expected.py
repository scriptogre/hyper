from collections.abc import Iterable
from hyper import component, replace_markers


@component
def DefaultSlot(_content: Iterable[str] | None = None, *, title: str):
    yield replace_markers(f"""\
<div class="card">
    <h2>‹ESCAPE:{title}›</h2>""")
    if _content is not None:
        yield from _content
    yield """</div>"""
