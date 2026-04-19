from hyper import html, escape


@html
def Greeting(*, name: str):
    yield f"""<h1>Hello, {escape(name)}!</h1>"""




