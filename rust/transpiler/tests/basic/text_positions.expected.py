from hyper import component


@component
def TextPositions(*, name: str):
    yield """\
Text before any element
<div>Inside div</div>
Text between elements
<span>Inside span</span>
Text after elements
More text
<p>
    Multiline
    text inside
    paragraph
</p>
Final text"""
