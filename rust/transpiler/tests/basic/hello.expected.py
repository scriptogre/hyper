from hyper import html


@html
def Hello():
    yield """<h1>Hello World</h1>"""
