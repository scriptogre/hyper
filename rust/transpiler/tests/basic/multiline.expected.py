from hyper import escape, replace_markers

def Multiline(data: dict, items: list) -> str:
    _parts = []
    config = {
    "key1": "value1",
    "key2": "value2",
    "key3": "value3"
}
    _parts.append(f"""<span>‹ESCAPE:{config}›</span>""")
    values = [
    "item1",
    "item2",
    "item3"
]
    _parts.append(f"""<span>‹ESCAPE:{values}›</span>""")
    result = some_function(
    arg1="value1",
    arg2="value2",
    arg3="value3"
)
    _parts.append(f"""<span>‹ESCAPE:{result}›</span>""")
    squares = [
    x ** 2
    for x in range(10)
    if x % 2 == 0
]
    _parts.append(f"""<span>‹ESCAPE:{squares}›</span>""")
    processed = (data
    .get('items', [])
    .copy()
)
    _parts.append(f"""<span>‹ESCAPE:{processed}›</span>""")
    return replace_markers("".join(_parts))
