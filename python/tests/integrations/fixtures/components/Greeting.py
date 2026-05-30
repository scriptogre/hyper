from hyper import html, escape


@html
def Greeting(
        *,
        name: str,
):
    yield f"""<p>Hello, {escape(name)}!</p>"""
