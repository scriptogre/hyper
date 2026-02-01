from hyper import component, replace_markers


@component
def WithSlot(*, title: str):
    # <{Card}>
    def _card():
        yield replace_markers(f"""\
    <h2>‹ESCAPE:{title}›</h2>
    <p>Card content goes here</p>""")
    yield from Card(_card())
    # </{Card}>
