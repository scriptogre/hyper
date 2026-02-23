from hyper import html, spread_attrs


@html
def Spread(*, attrs: dict = {"href": "https://example.com", "target": "_blank"}):
    yield f"""<a{spread_attrs(attrs)}>Link</a>"""
