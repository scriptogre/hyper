from hyper import component, replace_markers


@component
def Simple(*, title: str, count: int):
    yield replace_markers(f"""\
<div>
    <h1>‹ESCAPE:{title}›</h1>
    <p>Count: ‹ESCAPE:{count}›</p>
</div>""")
