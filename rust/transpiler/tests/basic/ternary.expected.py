from hyper import html, replace_markers


@html
def Ternary(*, count: int):
    yield replace_markers(f"""<span>‹ESCAPE:{count}› ‹ESCAPE:{"item" if count == 1 else "items"}›</span>""")
