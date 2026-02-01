from hyper import component, replace_markers


@component
def Comprehension(*, items: list[str]):
    yield replace_markers(f"""\
<ul>
    ‹ESCAPE:{[f"<li>{item}</li>" for item in items]}›
</ul>""")
