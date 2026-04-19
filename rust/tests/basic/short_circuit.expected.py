from hyper import html, escape


@html
def ShortCircuit(*, show_warning: bool, message: str):

    yield """<div>"""

    if show_warning:
        yield f"""\
<p class="warning">{escape(message)}</p>
    """

    yield """</div>"""

