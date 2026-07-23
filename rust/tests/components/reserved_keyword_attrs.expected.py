from hyperhtml import component, escape, render_class


@component
def ReservedKeywordAttrs(
        *,
        class_: str = "",
        type: str = "button",
):
    yield f"""<button class="{render_class(class_)}" type="{escape(type)}">"""
    yield from Icon.stream(class_="icon", type="svg")
    yield """</button>"""
