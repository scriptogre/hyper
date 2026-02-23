from hyper import html, render_class


@html
def ClassList(*, is_active: bool):

    _class = ["btn", "btn-primary", {"active": is_active}]
    yield f"""<button class="{render_class(_class)}">Click</button>"""
