from collections.abc import Iterable
from hyper import component


@component
def NamedSlots(
    _content: Iterable[str] | None = None,
    _sidebar: Iterable[str] | None = None
):
    yield """\
<div class="layout">
    <aside>"""
    if _sidebar is not None:
        yield from _sidebar
    yield """\
</aside>
    <main>"""
    if _content is not None:
        yield from _content
    yield """\
</main>
</div>"""
