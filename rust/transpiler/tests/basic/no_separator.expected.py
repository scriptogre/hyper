from hyper import component


@component
def NoSeparator():
    yield """\
<div>No separator, just content</div>
<span>More content</span>"""
