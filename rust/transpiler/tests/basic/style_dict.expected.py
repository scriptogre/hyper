from hyper import html, replace_markers


@html
def StyleDict():
    style = {"color": "red", "font-weight": "bold"}
    yield replace_markers(f"""<p style=‹STYLE:{style}›>Important</p>""")
