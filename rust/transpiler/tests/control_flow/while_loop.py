from hyper import escape, replace_markers

def WhileLoop(count: int) -> str:
    _parts = []
    _parts.append("<div>")
    while count > 0:
        _parts.append(f"""<p>Count: ‹ESCAPE:{count}›</p>""")
    _parts.append("</div>")
    return replace_markers("".join(_parts))
