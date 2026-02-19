from hyper import html, replace_markers


@html
def WhileLoop(*, count: int):

    yield "<div>"

    while count > 0:
        yield replace_markers(f"""\
<p>Count: ‹ESCAPE:{count}›</p>
    """)

    yield "</div>"

