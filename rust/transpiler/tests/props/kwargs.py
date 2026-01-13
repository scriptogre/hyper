from hyper import escape, replace_markers

def Kwargs(label: str, **attrs: dict) -> str:
    _parts = []
    _parts.append(f"""<button attrs=‹SPREAD:{attrs}›>‹ESCAPE:{label}›</button>""")
    return replace_markers("".join(_parts))
