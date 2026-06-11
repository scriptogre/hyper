from hyper import html, escape, render_class


@html
def ReservedKeywordAttrs(
        *,
        class_: str = "",
        type_: str = "button",
):
    yield f"""<button class="{render_class(class_)}" type="{escape(type_)}">"""
    yield from Icon(class_="icon", type_="svg")
    yield """</button>"""
