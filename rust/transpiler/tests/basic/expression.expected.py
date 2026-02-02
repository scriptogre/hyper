from hyper import component, replace_markers, escape


@component
def Expression(*, name: str):
    yield replace_markers(f"""<h1>Hello ‹ESCAPE:{name}›</h1>""")
