from hyper import html, render_attr


@html
def Spread(*, attrs: dict = {"href": "https://example.com", "target": "_blank"}):
    yield f"""<a{render_attr("attrs", attrs)}>Link</a>"""
