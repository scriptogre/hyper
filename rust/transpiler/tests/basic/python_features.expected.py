from hyper import html, escape


@html
def PythonFeatures(*, items: list, data: dict, value: int):

    # Walrus operator

    if (n := len(items)) > 0:
        yield f"""<span>Found {escape(n)} items</span>"""

    # Lambda (assigned to variable first)

    sorter = lambda x: x.lower()

    sorted_items = sorted(items, key=sorter)
    yield f"""<span>{escape(sorted_items)}</span>"""

    # Unpacking in expression

    first, *rest = items
    yield f"""<span>First: {escape(first)}, Rest: {escape(rest)}</span>"""

    # Dictionary merge (Python 3.9+)

    merged = {**data, "extra": "value"}
    yield f"""<span>{escape(merged)}</span>"""

    # Conditional expression (ternary)
    yield f"""<span>{escape(value if value > 0 else -value)}</span>"""

    # Nested ternary
    yield f"""<span>{escape("positive" if value > 0 else "zero" if value == 0 else "negative")}</span>"""

    # F-string style expressions
    yield f"""\
<span>{escape(value=)}</span>
<span>{escape(items!r)}</span>"""

    # Star unpacking in function call

    args = [1, 2, 3]
    yield f"""<span>{escape(max(*args))}</span>"""

    # Keyword unpacking

    kwargs = {"sep": ", ", "end": "!"}

    result = "hello"
    yield f"""<span>{escape(result)}</span>"""
