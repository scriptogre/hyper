from hyper import html, render_data, render_aria


@html
def DataAria(*, is_hidden: bool):

    data = {"user-id": 123, "role": "admin"}

    aria = {"label": "Close", "hidden": is_hidden}
    yield f"""<div{render_data(data)}{render_aria(aria)}>Content</div>"""
