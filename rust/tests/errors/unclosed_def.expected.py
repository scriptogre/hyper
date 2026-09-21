from hyper import component, escape


@component
def UnclosedDef():
    def greet(name: str):
        yield f"""<h1>Hello, {escape(name)}!</h1>"""
