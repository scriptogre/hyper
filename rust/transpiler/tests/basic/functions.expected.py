def Functions(*, name: str):
    def greet(who: str):
        yield replace_markers(f"""<h1>Hello, ‹ESCAPE:{who}›!</h1>""")
    def make_badge(text: str, color: str = "blue") -> str:
        yield replace_markers(f"""<span class="badge badge-{color}">‹ESCAPE:{text}›</span>""")
    from hyper import component, replace_markers, escape


@component
async def fetch_and_render(url: str):
        yield replace_markers(f"""<div class="loading">Fetching ‹ESCAPE:{url}›...</div>""")
    def render_list(items: list, title: str = "List"):
        yield "<div class=\"list-container\">"
        yield "<h2>"
        yield escape(title)
        yield "</h2>"
        if items:
            yield "<ul>"
            for item in items:
                yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
            yield "</ul>"
        else:
            yield """<p>No items</p>"""
        yield "</div>"
    greet(name)
    yield replace_markers(f"""‹ESCAPE:{make_badge("Admin", "red")}›""")
