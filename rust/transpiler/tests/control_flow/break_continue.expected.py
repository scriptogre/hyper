from hyper import component, replace_markers


@component
def BreakContinue(*, items: list, limit: int):
    # Break in for loop
    yield """<ul>"""
    for item in items:
        if item == "stop":
            break
        yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
    yield """</ul>"""

    # Continue in for loop
    yield """<ul>"""
    for item in items:
        if item.startswith("_"):
            continue
        yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
    yield """</ul>"""

    # Break in while loop
    count = 0
    while True:
        if count >= limit:
            break
        yield replace_markers(f"""<span>‹ESCAPE:{count}›</span>""")
        count = count + 1

    # Nested break
    for outer in items:
        for inner in items:
            if inner == outer:
                break
            yield replace_markers(f"""<span>‹ESCAPE:{outer}›-‹ESCAPE:{inner}›</span>""")
