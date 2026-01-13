from hyper import escape, replace_markers

def TryExcept(data: dict, risky_func: object) -> str:
    _parts = []
    try:
        _parts.append(f"""<span>‹ESCAPE:{data['missing']}›</span>""")
    except KeyError::
        _parts.append("""<span>Key not found</span>""")
    try:
        _parts.append(f"""<span>‹ESCAPE:{risky_func()}›</span>""")
    except ValueError as e::
        _parts.append(f"""<span>Value error: ‹ESCAPE:{e}›</span>""")
    except TypeError as e::
        _parts.append(f"""<span>Type error: ‹ESCAPE:{e}›</span>""")
    except Exception as e::
        _parts.append(f"""<span>Unknown error: ‹ESCAPE:{e}›</span>""")
    try:
        result = data['key']
    except KeyError::
        _parts.append("""<span>Missing</span>""")
    else:
        _parts.append(f"""<span>Found: ‹ESCAPE:{result}›</span>""")
    try:
        _parts.append(f"""<span>‹ESCAPE:{data['value']}›</span>""")
    except:
        _parts.append("""<span>Error</span>""")
    finally:
        _parts.append("""<span>Cleanup complete</span>""")
    try:
        value = data['key']
    except KeyError::
        _parts.append("""<span>Not found</span>""")
        value = 'default'
    else:
        _parts.append("""<span>Success</span>""")
    finally:
        _parts.append(f"""<span>Done: ‹ESCAPE:{value}›</span>""")
    return replace_markers("".join(_parts))
