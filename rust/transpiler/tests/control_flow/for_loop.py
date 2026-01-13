from hyper import escape, replace_markers

def ForLoop(items: list[str]) -> str:
    _parts = []
    _parts.append("<ul>")
    for item in items:
        _parts.append(f"""<li>‹ESCAPE:{item}›</li>""")
    _parts.append("</ul>")
    return replace_markers("".join(_parts))
