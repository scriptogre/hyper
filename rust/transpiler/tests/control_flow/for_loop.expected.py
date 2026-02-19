from hyper import html, replace_markers


@html
def ForLoop(*, items: list[str]):

    yield "<ul>"

    for item in items:
        yield replace_markers(f"""\
<li>‹ESCAPE:{item}›</li>
    """)

    yield "</ul>"
