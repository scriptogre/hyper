from hyperhtml import component


@component
def UnexpectedEnd():
    yield """<div>Content</div>"""
