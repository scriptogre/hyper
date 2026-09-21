from hyper import component, escape, render_class


@component
def ReservedKeywordAttrs(
        *,
        class_: str = "",
        type: str = "button",
):
    yield f"""<button class="{render_class(class_)}" type="{escape(type)}">"""
    yield from Icon(class_="icon", type="svg").render(stream=True)
    yield """</button>"""
