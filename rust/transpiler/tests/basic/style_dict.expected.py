from hyper import html, render_style


@html
def StyleDict():
    style = {"color": "red", "font-weight": "bold"}
    yield f"""<p style="{render_style(style)}">Important</p>"""
