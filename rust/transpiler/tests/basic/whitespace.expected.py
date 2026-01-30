def Whitespace(name: str) -> str:
    _parts = []
    _parts.append("""<div>After blank lines</div><span>Between blank lines</span><p>More content</p>""")
    return "".join(_parts)
