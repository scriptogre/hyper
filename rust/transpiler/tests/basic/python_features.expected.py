from hyper import escape, replace_markers

def PythonFeatures(items: list, data: dict, value: int) -> str:
    _parts = []
    if (n := len(items)) > 0:
        _parts.append(f"""<span>Found ‹ESCAPE:{n}› items</span>""")
    sorter = lambda x: x.lower()
    sorted_items = sorted(items, key=sorter)
    _parts.append(f"""<span>‹ESCAPE:{sorted_items}›</span>""")
    first, *rest = items
    _parts.append(f"""<span>First: ‹ESCAPE:{first}›, Rest: ‹ESCAPE:{rest}›</span>""")
    merged = {**data, "extra": "value"}
    _parts.append(f"""<span>‹ESCAPE:{merged}›</span><span>‹ESCAPE:{value if value > 0 else -value}›</span><span>‹ESCAPE:{"positive" if value > 0 else "zero" if value == 0 else "negative"}›</span><span>‹ESCAPE:{value=}›</span><span>‹ESCAPE:{items!r}›</span>""")
    args = [1, 2, 3]
    _parts.append(f"""<span>‹ESCAPE:{max(*args)}›</span>""")
    kwargs = {"sep": ", ", "end": "!"}
    result = "hello"
    _parts.append(f"""<span>‹ESCAPE:{result}›</span>""")
    return replace_markers("".join(_parts))
