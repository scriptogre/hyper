from hyper import html, replace_markers


@html
def Expression(*, name: str):
    yield replace_markers(f"""<h1>Hello ‹ESCAPE:{name}›</h1>""")
