from hyper import component, replace_markers, escape


@component
def PythonFeatures(*, items: list, data: dict, value: int):
    if (n := len(items)) > 0:
        yield replace_markers(f"""<span>Found ‹ESCAPE:{n}› items</span>""")
    sorter = lambda x: x.lower()
    sorted_items = sorted(items, key=sorter)
    yield replace_markers(f"""<span>‹ESCAPE:{sorted_items}›</span>""")
    first, *rest = items
    yield replace_markers(f"""<span>First: ‹ESCAPE:{first}›, Rest: ‹ESCAPE:{rest}›</span>""")
    merged = {**data, "extra": "value"}
    yield replace_markers(f"""<span>‹ESCAPE:{merged}›</span><span>‹ESCAPE:{value if value > 0 else -value}›</span><span>‹ESCAPE:{"positive" if value > 0 else "zero" if value == 0 else "negative"}›</span><span>‹ESCAPE:{value=}›</span><span>‹ESCAPE:{items!r}›</span>""")
    args = [1, 2, 3]
    yield replace_markers(f"""<span>‹ESCAPE:{max(*args)}›</span>""")
    kwargs = {"sep": ", ", "end": "!"}
    result = "hello"
    yield replace_markers(f"""<span>‹ESCAPE:{result}›</span>""")
