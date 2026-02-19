from hyper import html, replace_markers


@html
def ShortCircuit(*, show_warning: bool, message: str):
    yield replace_markers(f"""\
<div>
    ‹ESCAPE:{show_warning and <p class="warning">{message}</p>}›
</div>""")
