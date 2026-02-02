from hyper import component, replace_markers, escape


@component
def MatchGuards(*, value: int, data: dict):
    match value:
        case x if x < 0:
            yield replace_markers(f"""<span>Negative: ‹ESCAPE:{x}›</span>""")
        case x if x == 0:
            yield """<span>Zero</span>"""
        case x if x > 100:
            yield replace_markers(f"""<span>Large: ‹ESCAPE:{x}›</span>""")
        case x:
            yield replace_markers(f"""<span>Normal: ‹ESCAPE:{x}›</span>""")
    match data:
        case {"type": "user", "admin": True}:
            yield """<span>Admin user</span>"""
        case {"type": "user", "admin": False}:
            yield """<span>Regular user</span>"""
        case {"type": t} if t.startswith("system"):
            yield replace_markers(f"""<span>System: ‹ESCAPE:{t}›</span>""")
        case _:
            yield """<span>Unknown</span>"""
