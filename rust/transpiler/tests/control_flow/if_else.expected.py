def IfElse(is_admin: bool) -> str:
    _parts = []
    _parts.append("<nav>")
    if is_admin:
        _parts.append("""<a href="/admin">Admin</a>""")
    else:
        _parts.append("""<a href="/account">Account</a>""")
    _parts.append("</nav>")
    return "".join(_parts)
