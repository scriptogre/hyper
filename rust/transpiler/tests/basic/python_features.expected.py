from hyper import component, replace_markers


@component
def PythonFeatures(*, items: list, data: dict, value: int):
    # Walrus operator
    if (n := len(items)) > 0:
        yield replace_markers(f"""<span>Found ‹ESCAPE:{n}› items</span>""")

    # Lambda (assigned to variable first)
    sorter = lambda x: x.lower()
    sorted_items = sorted(items, key=sorter)
    yield replace_markers(f"""<span>‹ESCAPE:{sorted_items}›</span>""")

    # Unpacking in expression
    first, *rest = items
    yield replace_markers(f"""<span>First: ‹ESCAPE:{first}›, Rest: ‹ESCAPE:{rest}›</span>""")

    # Dictionary merge (Python 3.9+)
    merged = {**data, "extra": "value"}
    yield replace_markers(f"""<span>‹ESCAPE:{merged}›</span>""")

    # Conditional expression (ternary)
    yield replace_markers(f"""<span>‹ESCAPE:{value if value > 0 else -value}›</span>""")

    # Nested ternary
    yield replace_markers(f"""<span>‹ESCAPE:{"positive" if value > 0 else "zero" if value == 0 else "negative"}›</span>""")

    # F-string style expressions
    yield replace_markers(f"""\
<span>‹ESCAPE:{value=}›</span>
<span>‹ESCAPE:{items!r}›</span>""")

    # Star unpacking in function call
    args = [1, 2, 3]
    yield replace_markers(f"""<span>‹ESCAPE:{max(*args)}›</span>""")

    # Keyword unpacking
    kwargs = {"sep": ", ", "end": "!"}
    result = "hello"
    yield replace_markers(f"""<span>‹ESCAPE:{result}›</span>""")
