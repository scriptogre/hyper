from hyper import component, replace_markers, escape


@component
def Comprehension(*, items: list[str]):
    yield replace_markers(f"""<ul>‹ESCAPE:{[<li>{item}</li> for item in items]}›</ul>""")
