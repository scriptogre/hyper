from hyper import component, replace_markers


@component
def Defaults(*, name: str = "World", count: int = 0):
    yield replace_markers(f"""\
<h1>Hello ‹ESCAPE:{name}›</h1>
<p>Count: ‹ESCAPE:{count}›</p>""")
