def Whitespace(name: str) -> str:
    _parts = []

    _parts.append("""<div>After blank lines</div>""")

    _parts.append("""<span>Between blank lines</span>""")

    _parts.append("""<p>More content</p>""")
    return "".join(_parts)
