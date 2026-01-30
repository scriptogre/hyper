from hyper import escape, replace_markers

def WithSlot(title: str) -> str:
    _parts = []
    _child_parts = f"""<h2>‹ESCAPE:{title}›</h2>
    <p>Card content goes here</p>"""
    _parts.append(Card(_children=_child_parts))
    return replace_markers("".join(_parts))
