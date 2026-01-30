from hyper import escape, replace_markers

def Comprehensive(name: str, count: int = 0, is_active: bool = True) -> str:
    _parts = []
    _parts.append("<div class=\"container\">")
    _parts.append("<span>")
    _parts.append("Plain text")
    _parts.append("</span>")
    _parts.append("<p>")
    _parts.append(escape(name))
    _parts.append("</p>")
    _parts.append("<p>")
    _parts.append("Count: ")
    _parts.append(escape(count + 1))
    _parts.append("</p>")
    if is_active:
        _parts.append("""<span>Active</span>""")
    elif count > 0:
        _parts.append("""<span>Has count</span>""")
    else:
        _parts.append("""<span>Inactive</span>""")
    for item in items:
        _parts.append(f"""<li>‹ESCAPE:{item}›</li>""")
    _parts.append("</div>")
    return replace_markers("".join(_parts))
