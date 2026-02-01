from hyper import component, replace_markers


@component
def ClassList(*, is_active: bool):
    _class = ["btn", "btn-primary", {"active": is_active}]
    yield replace_markers(f"""<button class=‹CLASS:{_class}›>Click</button>""")
