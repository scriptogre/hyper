from hyper import component


@component
def MissingEnd():
    yield """<div>"""
    if show:
        yield """<p>Hello</p>"""
    yield """</div>"""
