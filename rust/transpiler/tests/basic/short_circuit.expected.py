from hyper import html, replace_markers


@html
def ShortCircuit(*, show_warning: bool, message: str):

    yield "<div>"

    if show_warning:
        yield replace_markers(f"""\
<p class="warning">‹ESCAPE:{message}›</p>
    """)

    yield "</div>"

