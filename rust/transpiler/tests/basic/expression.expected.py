from hyper import html, escape


@html
def Expression(*, name: str):
    yield f"""<h1>Hello {escape(name)}</h1>"""
