from hyper import html, escape, render_class


@html
def ReservedKeywords():

    class_ = "primary"

    type_ = "button"
    yield f"""<button class="{render_class(class_)}" type="{escape(type_)}">Click</button>"""
