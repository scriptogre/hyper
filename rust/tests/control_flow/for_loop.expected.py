from hyper import html, escape


@html
def ForLoop(*, items: list[str]):

    yield """<ul>"""

    for item in items:
        yield f"""\
<li>{escape(item)}</li>
    """

    yield """</ul>"""
