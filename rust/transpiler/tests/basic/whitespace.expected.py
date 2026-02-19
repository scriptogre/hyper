from hyper import html


@html
def Whitespace(*, name: str):
    yield """\
<div>After blank lines</div>

<span>Between blank lines</span>

<p>More content</p>"""


