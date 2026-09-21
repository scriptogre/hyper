from hyper import component


@component
def UnclosedIf():
    if show:
        yield """<p>Hello</p>"""
    else:
        yield """<p>Goodbye</p>"""
