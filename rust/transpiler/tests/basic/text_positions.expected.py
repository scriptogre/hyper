from hyper import component


@component
def TextPositions(*, name: str):
    yield """Text before any element<div>Inside div</div>Text between elements<span>Inside span</span>Text after elementsMore text<p>Multilinetext insideparagraph</p>Final text"""
