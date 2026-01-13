def MissingEnd() -> str:
    _parts = []
    _parts.append("<div>")
    if show:
        _parts.append("""<p>Hello</p>""")
    _parts.append("</div>")
    return "".join(_parts)
