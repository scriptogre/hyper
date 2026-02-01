from hyper import replace_markers

def DataAria(is_hidden: bool) -> str:
    _parts = []
    data = {"user-id": 123, "role": "admin"}
    aria = {"label": "Close", "hidden": is_hidden}
    _parts.append(f"""<div data=‹DATA:{data}› aria=‹ARIA:{aria}›>Content</div>""")
    return replace_markers("".join(_parts))
