from hyper import escape, replace_markers

def BreakContinue(items: list, limit: int) -> str:
    _parts = []
    _parts.append("<ul>")
    for item in items:
        if item == "stop":
            break
        _parts.append(f"""<li>‹ESCAPE:{item}›</li>""")
    _parts.append("</ul>")
    _parts.append("<ul>")
    for item in items:
        if item.startswith("_"):
            continue
        _parts.append(f"""<li>‹ESCAPE:{item}›</li>""")
    _parts.append("</ul>")
    count = 0
    while True:
        if count >= limit:
            break
        _parts.append(f"""<span>‹ESCAPE:{count}›</span>""")
        count = count + 1
    for outer in items:
        for inner in items:
            if inner == outer:
                break
            _parts.append(f"""<span>‹ESCAPE:{outer}›-‹ESCAPE:{inner}›</span>""")
    return replace_markers("".join(_parts))
