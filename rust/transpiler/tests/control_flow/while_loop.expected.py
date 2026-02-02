from hyper import component, replace_markers, escape


@component
def WhileLoop(*, count: int):
    yield "<div>"
    while count > 0:
        yield replace_markers(f"""<p>Count: ‹ESCAPE:{count}›</p>""")
    yield "</div>"
