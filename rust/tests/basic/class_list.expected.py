from hyper import html, render_class


@html
def ClassList(*, is_active: bool):

    class_ = ["btn", "btn-primary", {"active": is_active}]
    yield f"""<button class="{render_class(class_)}">Click</button>"""
