from hyper import html, replace_markers


@html
def Comprehension(*, items: list[str]):
    yield replace_markers(f"""\
<ul>
    ‹ESCAPE:{[<li>{item}</li> for item in items]}›
</ul>""")
