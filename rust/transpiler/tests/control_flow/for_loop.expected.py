from hyper import component, replace_markers


@component
def ForLoop(*, items: list[str]):
    yield """<ul>"""
    for item in items:
        yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
    yield """</ul>"""
