from hyper import component, replace_markers, escape


@component
def BreakContinue(*, items: list, limit: int):
    yield "<ul>"
    for item in items:
        if item == "stop":
            break
        yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
    yield "</ul>"
    yield "<ul>"
    for item in items:
        if item.startswith("_"):
            continue
        yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
    yield "</ul>"
    count = 0
    while True:
        if count >= limit:
            break
        yield replace_markers(f"""<span>‹ESCAPE:{count}›</span>""")
        count = count + 1
    for outer in items:
        for inner in items:
            if inner == outer:
                break
            yield replace_markers(f"""<span>‹ESCAPE:{outer}›-‹ESCAPE:{inner}›</span>""")
