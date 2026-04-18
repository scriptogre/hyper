from collections.abc import Iterable
from hyper import html, escape


@html
def Decorators(_content: Iterable[str] | None = None, *, items: list):

    # Simple decorator

    @fragment
    def Badge(text: str):
        yield f"""<span class="badge">{escape(text)}</span>"""

    # Multiple decorators

    @cache
    @fragment
    def CachedList(items: list):

        yield "<ul>"

        for item in items:
            yield f"""\
<li>{escape(item)}</li>
        """

        yield "</ul>"


    # Decorator with arguments

    @fragment(name="card")
    def Card(title: str):

        yield "<div class=\"card\">"
        yield f"""\
<h2>{escape(title)}</h2>
        """
        if _content is not None:
            yield from _content

        yield "</div>"


    # Use decorated functions
    yield f"""\
{escape(Badge("New"))}
{escape(CachedList(items))}"""
