from hyper import html, escape


@html
def WithSlot(*, title: str):

    # <{Card}>
    def _card():
        yield f"""\
<h2>{escape(title)}</h2>
    <p>Card content goes here</p>"""
    yield from Card(_card())
    # </{Card}>

