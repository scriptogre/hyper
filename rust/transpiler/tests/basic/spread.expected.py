from hyper import replace_markers

def Spread(attrs: dict = {"href": "https://example.com", "target": "_blank"}) -> str:
    _parts = []
    _parts.append(f"""<a attrs=‹SPREAD:{attrs}›>Link</a>""")
    return replace_markers("".join(_parts))
