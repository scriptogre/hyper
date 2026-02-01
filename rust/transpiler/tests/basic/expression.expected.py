from hyper import component, replace_markers


@component
def Expression(*, name: str):
    yield replace_markers(f"""<h1>Hello ‹ESCAPE:{name}›</h1>""")
