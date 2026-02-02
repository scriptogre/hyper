from hyper import component, replace_markers, escape


@component
def PropsEdgeCases(*, simple: str, with_default: str = "default value", none_default: str | None = None, empty_string: str = "", zero_default: int = 0, false_default: bool = False, empty_list: list = [], empty_dict: dict = {}, complex_default: dict = {"key": "value"}, *args: tuple, **kwargs: Any):
    """This is a component docstring."""
    yield replace_markers(f"""<div><span>‹ESCAPE:{simple}›</span><span>‹ESCAPE:{with_default}›</span><span>‹ESCAPE:{none_default or "none"}›</span></div>""")
