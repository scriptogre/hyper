from hyper import replace_markers

def Spread() -> str:
    _parts = []
    attrs = {"href": "https://example.com", "target": "_blank"}
    _parts.append(f"""<a attrs=‹SPREAD:{attrs}›>Link</a>""")
    return replace_markers("".join(_parts))
