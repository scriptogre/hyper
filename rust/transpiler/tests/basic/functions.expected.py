from hyper import html, replace_markers


@html
def Functions(*, name: str):
    @html
    def Greet(who: str):
        yield replace_markers(f"""<h1>Hello, ‹ESCAPE:{who}›!</h1>""")
    @html
    def Badge(text: str, color: str = "blue"):
        yield replace_markers(f"""<span class="badge badge-‹ESCAPE:{color}›">‹ESCAPE:{text}›</span>""")
    @html
    def List(items: list, title: str = "Items"):
        yield "<div class=\"list-container\">"
        yield replace_markers(f"""<h2>‹ESCAPE:{title}›</h2>""")
        if items:
            yield "<ul>"
            for item in items:
                yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
            yield "</ul>"
        else:
            yield """<p>No items</p>"""
        yield "</div>"
    def format_name(n: str) -> str:
        return n.upper()

    yield from Greet(who=format_name(name))

    yield from Badge(text="Admin", color="red")

