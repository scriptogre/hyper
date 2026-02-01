from hyper import escape, replace_markers

def Simple(title: str, count: int) -> str:
    _parts = []
    _parts.append(f"""<div><h1>‹ESCAPE:{title}›</h1><p>Count: ‹ESCAPE:{count}›</p></div>""")
    return replace_markers("".join(_parts))
