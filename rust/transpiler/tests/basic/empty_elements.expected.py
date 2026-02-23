from hyper import html


@html
def EmptyElements():
    yield """\
<div></div>
<span></span>
<p></p>"""
