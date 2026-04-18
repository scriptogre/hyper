from hyper import html


@html
def NoSeparator():
    yield """<div>No separator, just content</div><span>More content</span>"""
