from hyper import replace_markers

def Boolean(is_disabled: bool) -> str:
    _parts = []
    _parts.append(f"""<button disabled=‹BOOL:{is_disabled}›>Submit</button>""")
    return replace_markers("".join(_parts))
