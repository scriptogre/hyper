from collections.abc import Iterable
from hyper import component, replace_markers


@component
def Decorators(_content: Iterable[str] | None = None, *, items: list):
    # Simple decorator
    @fragment
    def Badge(text: str):
        yield replace_markers(f"""<span class="badge">‹ESCAPE:{text}›</span>""")

    # Multiple decorators
    @cache
    @fragment
    def CachedList(items: list):
        yield """<ul>"""
        for item in items:
            yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
        yield """</ul>"""

    # Decorator with arguments
    @fragment(name="card")
    def Card(_content: Iterable[str] | None = None, *, title: str):
        yield replace_markers(f"""\
<div class="card">
    <h2>‹ESCAPE:{title}›</h2>""")
        if _content is not None:
            yield from _content
        yield """</div>"""

    # Use decorated functions
    yield replace_markers(f"""‹ESCAPE:{"".join(Badge("New"))}›‹ESCAPE:{"".join(CachedList(items))}›""")
