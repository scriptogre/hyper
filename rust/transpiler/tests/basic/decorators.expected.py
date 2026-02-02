@fragment
@cache
@fragment
@fragment(name="card")
from collections.abc import Iterable
from hyper import component, replace_markers, escape


@component
def Decorators(_content: Iterable[str] | None = None, *, items: list):
    def Badge(text: str):
        yield replace_markers(f"""<span class="badge">‹ESCAPE:{text}›</span>""")
    def CachedList(items: list):
        yield "<ul>"
        for item in items:
            yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
        yield "</ul>"
    def Card(title: str):
        yield "<div class=\"card\">"
        yield "<h2>"
        yield escape(title)
        yield "</h2>"
        if _content is not None:
            yield from _content
        yield "</div>"
    yield replace_markers(f"""‹ESCAPE:{Badge("New")}›‹ESCAPE:{CachedList(items)}›""")
