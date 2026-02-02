from hyper import component, replace_markers, escape


@component
def Multiline(*, data: dict, items: list):
    config = {
    "key1": "value1",
    "key2": "value2",
    "key3": "value3"
}
    yield replace_markers(f"""<span>‹ESCAPE:{config}›</span>""")
    values = [
    "item1",
    "item2",
    "item3"
]
    yield replace_markers(f"""<span>‹ESCAPE:{values}›</span>""")
    result = some_function(
    arg1="value1",
    arg2="value2",
    arg3="value3"
)
    yield replace_markers(f"""<span>‹ESCAPE:{result}›</span>""")
    squares = [
    x ** 2
    for x in range(10)
    if x % 2 == 0
]
    yield replace_markers(f"""<span>‹ESCAPE:{squares}›</span>""")
    processed = (data
    .get('items', [])
    .copy()
)
    yield replace_markers(f"""<span>‹ESCAPE:{processed}›</span>""")
