from hyper import html, render_attr


@html
def Boolean(*, is_disabled: bool):
    yield f"""<button{render_attr("disabled", is_disabled)}>Submit</button>"""
