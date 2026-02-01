from hyper import component, replace_markers


@component
def ShortCircuit(*, show_warning: bool, message: str):
    yield replace_markers(f"""\
<div>
    ‹ESCAPE:{show_warning and f'<p class="warning">{message}</p>'}›
</div>""")
