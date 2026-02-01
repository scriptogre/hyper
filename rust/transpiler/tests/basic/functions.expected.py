from hyper import component, replace_markers


@component
def Functions(*, name: str):
    # Simple function
    def greet(who: str):
        yield replace_markers(f"""<h1>Hello, ‹ESCAPE:{who}›!</h1>""")

    # Function with return type
    def make_badge(text: str, color: str = "blue"):
        yield replace_markers(f"""<span class="badge badge-‹ESCAPE:{color}›">‹ESCAPE:{text}›</span>""")

    # Async function
    async def fetch_and_render(url: str):
        yield replace_markers(f"""<div class="loading">Fetching ‹ESCAPE:{url}›...</div>""")

    # Function with complex body
    def render_list(items: list, title: str = "List"):
        yield replace_markers(f"""\
<div class="list-container">
    <h2>‹ESCAPE:{title}›</h2>""")
        if items:
            yield """<ul>"""
            for item in items:
                yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
            yield """</ul>"""
        else:
            yield """<p>No items</p>"""
        yield """</div>"""

    # Call the function
    yield from greet(name)
    yield replace_markers(f"""‹ESCAPE:{"".join(make_badge("Admin", "red"))}›""")
