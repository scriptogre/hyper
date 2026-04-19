from hyper import html, spread_attrs


@html
def Spread(*, attrs: dict = {"href": "https://example.com", "target": "_blank"}, extra: dict):

    # Spread on HTML element
    yield f"""<a{spread_attrs(attrs)}>External link</a>"""

    # Spread with other attributes
    yield f"""<div id="main"{spread_attrs(extra)} class="container">Content</div>"""
