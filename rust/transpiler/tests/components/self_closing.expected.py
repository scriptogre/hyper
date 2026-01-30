def SelfClosing(name: str, onClick: object, props: dict) -> str:
    _parts = []

    # Simple self-closing
    _parts.append(Button())

    # With attributes
    _parts.append(Button(label="Click me"))

    # With expression attributes
    _parts.append(Button(label=name, onClick=onClick))

    # With spread
    _parts.append(Button(**props))

    # Mixed
    _parts.append(Icon(name="star", size=24, **props))
    return "".join(_parts)
