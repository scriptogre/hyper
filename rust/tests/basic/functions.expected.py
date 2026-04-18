from hyper import html, escape


@html
def Functions(*, name: str):
    @html
    def Greet(who: str):
        yield f"""<h1>Hello, {escape(who)}!</h1>"""
    @html
    def Badge(text: str, color: str = "blue"):
        yield f"""<span class="badge badge-{escape(color)}">{escape(text)}</span>"""
    @html
    def List(items: list, title: str = "Items"):
        yield "<div class=\"list-container\">"
        yield f"""<h2>{escape(title)}</h2>"""
        if items:
            yield "<ul>"
            for item in items:
                yield f"""<li>{escape(item)}</li>"""
            yield "</ul>"
        else:
            yield """<p>No items</p>"""
        yield "</div>"
    def format_name(n: str) -> str:
        return n.upper()

    yield from Greet(who=format_name(name))

    yield from Badge(text="Admin", color="red")

