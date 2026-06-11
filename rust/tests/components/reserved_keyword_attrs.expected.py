from hyper import html, escape, render_class


@html
def ReservedKeywordAttrs(
        *,
        class_: str = "",
        type: str = "button",
):
    yield f"""<button class="{render_class(class_)}" type="{escape(type)}">"""
    yield from Icon(class_="icon", type="svg")
    yield """</button>"""
