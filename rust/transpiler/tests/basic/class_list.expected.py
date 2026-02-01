from hyper import replace_markers

def ClassList(is_active: bool) -> str:
    _parts = []
    _class = ["btn", "btn-primary", {"active": is_active}]
    _parts.append(f"""<button class=‹CLASS:{_class}›>Click</button>""")
    return replace_markers("".join(_parts))
