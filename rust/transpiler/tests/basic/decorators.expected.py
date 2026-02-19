@fragment
@cache
@fragment
@fragment(name="card")
from collections.abc import Iterable
from hyper import html, replace_markers


@html
def Decorators(_content: Iterable[str] | None = None, *, items: list):

    # Simple decorator

    def Badge(text: str):
        yield replace_markers(f"""<span class="badge">‹ESCAPE:{text}›</span>""")

    # Multiple decorators

    def CachedList(items: list):

        yield "<ul>"

        for item in items:
            yield replace_markers(f"""\
<li>‹ESCAPE:{item}›</li>
        """)

        yield "</ul>"


    # Decorator with arguments

    def Card(title: str):

        yield "<div class=\"card\">"
        yield replace_markers(f"""\
<h2>‹ESCAPE:{title}›</h2>
        """)
        if _content is not None:
            yield from _content

        yield "</div>"


    # Use decorated functions
    yield replace_markers(f"""\
‹ESCAPE:{Badge("New")}›
‹ESCAPE:{CachedList(items)}›""")
