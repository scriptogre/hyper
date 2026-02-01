from hyper import escape

def WithSlot(title: str) -> str:
    _parts = []
    _child_parts = []
    _child_parts.append("<h2>")
    _child_parts.append(escape(title))
    _child_parts.append("</h2>")
    _child_parts.append("<p>")
    _child_parts.append("Card content goes here")
    _child_parts.append("</p>")
    _parts.append(Card(_children="".join(_child_parts)))
    return "".join(_parts)
