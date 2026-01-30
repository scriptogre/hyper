def NoSeparator() -> str:
    _parts = []
    _parts.append("""<div>No separator, just content</div><span>More content</span>""")
    return "".join(_parts)
