from hyper import component, replace_markers, escape


@component
def Ternary(*, count: int):
    yield replace_markers(f"""<span>‹ESCAPE:{count}› ‹ESCAPE:{"item" if count == 1 else "items"}›</span>""")
