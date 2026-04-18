from hyper import html, escape


@html
def MatchGuards(*, value: int, data: dict):

    # Match with guards

    match value:
        case x if x < 0:
            yield f"""\
<span>Negative: {escape(x)}</span>
    """
        case x if x == 0:
            yield """\
<span>Zero</span>
    """
        case x if x > 100:
            yield f"""\
<span>Large: {escape(x)}</span>
    """
        case x:
            yield f"""<span>Normal: {escape(x)}</span>"""

    # Match with pattern guards

    match data:
        case {"type": "user", "admin": True}:
            yield """\
<span>Admin user</span>
    """
        case {"type": "user", "admin": False}:
            yield """\
<span>Regular user</span>
    """
        case {"type": t} if t.startswith("system"):
            yield f"""\
<span>System: {escape(t)}</span>
    """
        case _:
            yield """<span>Unknown</span>"""

