from hyper import html, escape


@html
def Multiline(*, data: dict, items: list):
    # Multiline dict literal
    config = {
        "key1": "value1",
        "key2": "value2",
        "key3": "value3"
    }
    yield f"""<span>{escape(config)}</span>"""
    # Multiline list literal
    values = [
        "item1",
        "item2",
        "item3"
    ]
    yield f"""<span>{escape(values)}</span>"""
    # Multiline function call
    result = some_function(
        arg1="value1",
        arg2="value2",
        arg3="value3"
    )
    yield f"""<span>{escape(result)}</span>"""
    # Multiline list comprehension
    squares = [
        x ** 2
        for x in range(10)
        if x % 2 == 0
    ]
    yield f"""<span>{escape(squares)}</span>"""
    # Chained method calls (each on own line)
    processed = (data
        .get('items', [])
        .copy()
    )
    yield f"""<span>{escape(processed)}</span>"""
