from hyper import component, replace_markers, escape


@component
def ComplexTypes(*, simple: str, with_default: str = "default", optional: str | None = None, items: list[str], mapping: dict[str, int], nested: list[dict[str, Any]], callback: Callable[[int], str], *args: tuple, **kwargs: Any):
    yield replace_markers(f"""<div><span>‹ESCAPE:{simple}›</span><span>‹ESCAPE:{with_default}›</span><span>‹ESCAPE:{optional or 'none'}›</span><span>‹ESCAPE:{len(items)}›</span><span>‹ESCAPE:{len(mapping)}›</span></div>""")
