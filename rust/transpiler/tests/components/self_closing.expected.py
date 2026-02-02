from hyper import component


@component
def SelfClosing(*, name: str, onClick: object, props: dict):
    yield from Button()
    yield from Button(label="Click me")
    yield from Button(label=name, onClick=onClick)
    yield from Button()
    yield from Icon(name="star", size=24, )
