from hyper import component


@component
def Hello():
    yield """<h1>Hello World</h1>"""
