from hyper import html, replace_markers


@html
def Spread(*, attrs: dict = {"href": "https://example.com", "target": "_blank"}):
    yield replace_markers(f"""<a attrs=‹SPREAD:{attrs}›>Link</a>""")
