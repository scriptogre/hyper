from typing import Any
from hyper import html, escape


@html
def PropsEdgeCases(*, simple: str, with_default: str = "default value", none_default: str | None = None, empty_string: str = "", zero_default: int = 0, false_default: bool = False, empty_list: list = [], empty_dict: dict = {}, complex_default: dict = {"key": "value"}, **kwargs: Any):
    # Docstring in header
    """This is a component docstring."""
    # Various parameter patterns
    yield f"""\
<div>
    <span>{escape(simple)}</span>
    <span>{escape(with_default)}</span>
    <span>{escape(none_default or "none")}</span>
</div>"""
