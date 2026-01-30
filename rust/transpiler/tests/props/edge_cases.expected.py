from typing import Any
from hyper import escape, replace_markers

def EdgeCases(
    simple: str,
    with_default: str = "default value",
    none_default: str | None = None,
    empty_string: str = "",
    zero_default: int = 0,
    false_default: bool = False,
    empty_list: list = None,
    empty_dict: dict = None,
    complex_default: dict = None,
    *args: tuple,
    **kwargs: Any,
) -> str:
    """This is a component docstring."""
    _parts = []
    if empty_list is None:
        empty_list = []
    if empty_dict is None:
        empty_dict = {}
    if complex_default is None:
        complex_default = {"key": "value"}
    _parts.append(f"""<div>
    <span>‹ESCAPE:{simple}›</span>
    <span>‹ESCAPE:{with_default}›</span>
    <span>‹ESCAPE:{none_default or "none"}›</span>
</div>""")
    return replace_markers("".join(_parts))
