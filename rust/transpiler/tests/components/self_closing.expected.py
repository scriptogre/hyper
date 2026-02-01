def SelfClosing(name: str, onClick: object, props: dict) -> str:
    _parts = []
    _parts.append(Button())
    _parts.append(Button(label="Click me"))
    _parts.append(Button(label=name, onClick=onClick))
    _parts.append(Button())
    _parts.append(Icon(name="star", size=24, ))
    return "".join(_parts)
