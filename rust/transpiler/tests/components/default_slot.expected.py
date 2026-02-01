from hyper import escape

def DefaultSlot(title: str, *, _children: str = "") -> str:
    _parts = []
    _parts.append("<div class=\"card\">")
    _parts.append("<h2>")
    _parts.append(escape(title))
    _parts.append("</h2>")
    _parts.append(_children)
    _parts.append("</div>")
    return "".join(_parts)
