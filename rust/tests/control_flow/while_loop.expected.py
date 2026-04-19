from hyper import html, escape


@html
def WhileLoop(*, count: int):

    yield """<div>"""

    while count > 0:
        yield f"""\
<p>Count: {escape(count)}</p>
    """

    yield """</div>"""

