from hyper import html


@html
def UnexpectedEnd():
    yield """<div>Content</div>"""
