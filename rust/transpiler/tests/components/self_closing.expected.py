from hyper import component


@component
def SelfClosing(*, name: str, onClick: object, props: dict):
    # Simple self-closing
    yield from Button()

    # With attributes
    yield from Button(label="Click me")

    # With expression attributes
    yield from Button(label=name, onClick=onClick)

    # With spread
    yield from Button(**props)

    # Mixed
    yield from Icon(name="star", size=24, **props)
