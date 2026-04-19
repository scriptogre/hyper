from hyper import html


@html
def SelfClosing(*, label: str, disabled: bool, variant: str = "primary"):

    # Simple self-closing

    yield from Button()

    # With static attribute

    yield from Button(label="Click me")

    # With expression attributes

    yield from Button(label=label, variant=variant)

    # With shorthand

    yield from Button(disabled=disabled)

    # Mixed

    yield from Icon(name="star", size=24, disabled=disabled)

