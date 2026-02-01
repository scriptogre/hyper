from hyper import component, replace_markers


@component
def StyleDict():
    style = {"color": "red", "font-weight": "bold"}
    yield replace_markers(f"""<p style=‹STYLE:{style}›>Important</p>""")
