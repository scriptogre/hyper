def TextPositions(name: str) -> str:
    _parts = []
    _parts.append("""Text before any element<div>Inside div</div>Text between elements<span>Inside span</span>Text after elementsMore text<p>Multilinetext insideparagraph</p>Final text""")
    return "".join(_parts)
