from hyper import escape, replace_markers

def Nested(title: str, items: list) -> str:
    _parts = []

    # Nested components
    _list_children = []
    for item in items:
        _list_children.append(ListItem(_children=f"‹ESCAPE:{item}›"))
    _card_body_children = List(_children="".join(_list_children))
    _card_header_children = f"<h2>‹ESCAPE:{title}›</h2>"
    _card_children = CardHeader(_children=_card_header_children) + CardBody(_children=_card_body_children)
    _parts.append(Card(_children=_card_children))

    # Component in control flow
    if title:
        _parts.append(Alert(type="info", _children=f"<span>‹ESCAPE:{title}›</span>"))

    # Components in loop
    for item in items:
        _parts.append(Badge(color="blue", _children=f"‹ESCAPE:{item}›"))
    return replace_markers("".join(_parts))
