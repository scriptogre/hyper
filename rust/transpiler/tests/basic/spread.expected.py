from hyper import component, replace_markers


@component
def Spread(*, attrs: dict = {"href": "https://example.com", "target": "_blank"}):
    yield replace_markers(f"""<a attrs=‹SPREAD:{attrs}›>Link</a>""")
