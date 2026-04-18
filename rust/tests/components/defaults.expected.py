from hyper import html, escape


@html
def Defaults(*, name: str = "World", count: int = 0):
    yield f"""\
<h1>Hello {escape(name)}</h1>
<p>Count: {escape(count)}</p>"""
