from hyper import component, replace_markers


@component
def Ternary(*, count: int):
    yield replace_markers(f"""<span>‹ESCAPE:{count}› ‹ESCAPE:{"item" if count == 1 else "items"}›</span>""")
