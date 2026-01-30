def Examples(user: dict, items: list, count: int = 0, is_active: bool = True) -> str:
    _parts = []
    if is_active:
        _parts.append("""<span>Active</span>""")
    return "".join(_parts)
