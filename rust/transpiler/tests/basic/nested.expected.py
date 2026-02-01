from hyper import component


@component
def Nested():
    yield """\
<div>
    <h1>Title</h1>
    <p>Paragraph</p>
</div>"""
