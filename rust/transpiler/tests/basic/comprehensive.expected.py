from hyper import component, replace_markers, escape


@component
def Comprehensive(*, name: str, count: int = 0, is_active: bool = True):
    yield "<div class=\"container\">"
    yield "<span>"
    yield "Plain text"
    yield "</span>"
    yield "<p>"
    yield escape(name)
    yield "</p>"
    yield "<p>"
    yield "Count: "
    yield escape(count + 1)
    yield "</p>"
    if is_active:
        yield """<span>Active</span>"""
    elif count > 0:
        yield """<span>Has count</span>"""
    else:
        yield """<span>Inactive</span>"""
    for item in items:
        yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
    yield "</div>"
