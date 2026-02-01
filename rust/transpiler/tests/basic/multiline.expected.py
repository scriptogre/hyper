from hyper import component, replace_markers


@component
def Multiline(*, data: dict, items: list):
    # Multiline dict literal
    config = {
        "key1": "value1",
        "key2": "value2",
        "key3": "value3"
    }
    yield replace_markers(f"""<span>‹ESCAPE:{config}›</span>""")

    # Multiline list literal
    values = [
        "item1",
        "item2",
        "item3"
    ]
    yield replace_markers(f"""<span>‹ESCAPE:{values}›</span>""")

    # Multiline function call
    result = some_function(
        arg1="value1",
        arg2="value2",
        arg3="value3"
    )
    yield replace_markers(f"""<span>‹ESCAPE:{result}›</span>""")

    # Multiline list comprehension
    squares = [
        x ** 2
        for x in range(10)
        if x % 2 == 0
    ]
    yield replace_markers(f"""<span>‹ESCAPE:{squares}›</span>""")

    # Chained method calls (each on own line)
    processed = (data
        .get('items', [])
        .copy()
    )
    yield replace_markers(f"""<span>‹ESCAPE:{processed}›</span>""")
