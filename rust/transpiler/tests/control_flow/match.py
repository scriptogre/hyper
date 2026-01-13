def Match(status: str) -> str:
    _parts = []
    match status:
        case "loading":
            _parts.append("""<p>Loading...</p>""")
        case "error":
            _parts.append("""<p>Error!</p>""")
        case _:
            _parts.append("""<p>Ready</p>""")
    return "".join(_parts)
