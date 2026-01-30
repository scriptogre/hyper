from hyper import replace_markers

def Spread(attrs: dict = None) -> str:
    _parts = []
    if attrs is None:
        attrs = {"href": "https://example.com", "target": "_blank"}
    _parts.append(f"""<a ‹SPREAD:{attrs}›>Link</a>""")
    return replace_markers("".join(_parts))
