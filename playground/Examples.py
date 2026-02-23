def Examples(user: dict, items: list, count: int = 0, is_active: bool = True):
    _parts = []

    _parts.append(f"""---""")

    # Simple if
    if is_active:
        _parts.append(f"""<span>Active {count}</span>""")  # Comment
    return ''.join(_parts)
