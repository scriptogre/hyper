from hyper import escape, replace_markers

def ShortCircuit(show_warning: bool, message: str) -> str:
    _parts = []
    _parts.append(f"""<div>‹ESCAPE:{show_warning and <p class="warning">{message}</p>}›</div>""")
    return replace_markers("".join(_parts))
