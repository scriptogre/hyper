from typing import Callable
from hyper import html


@html
def SelfClosing(*, name: str, on_click: Callable, disabled: bool):

    # Simple self-closing

    yield from Button()

    # With attributes

    yield from Button(label="Click me")

    # With expression attributes

    yield from Button(label=name, on_click=on_click)

    # With shorthand

    yield from Button(disabled=disabled)

    # Mixed

    yield from Icon(name="star", size=24, disabled=disabled)

