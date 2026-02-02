from hyper import component, replace_markers, escape


@component
def TryExcept(*, data: dict, risky_func: object):
    try:
        yield replace_markers(f"""<span>‹ESCAPE:{data['missing']}›</span>""")
    except KeyError::
        yield """<span>Key not found</span>"""
    try:
        yield replace_markers(f"""<span>‹ESCAPE:{risky_func()}›</span>""")
    except ValueError as e::
        yield replace_markers(f"""<span>Value error: ‹ESCAPE:{e}›</span>""")
    except TypeError as e::
        yield replace_markers(f"""<span>Type error: ‹ESCAPE:{e}›</span>""")
    except Exception as e::
        yield replace_markers(f"""<span>Unknown error: ‹ESCAPE:{e}›</span>""")
    try:
        result = data['key']
    except KeyError::
        yield """<span>Missing</span>"""
    else:
        yield replace_markers(f"""<span>Found: ‹ESCAPE:{result}›</span>""")
    try:
        yield replace_markers(f"""<span>‹ESCAPE:{data['value']}›</span>""")
    except:
        yield """<span>Error</span>"""
    finally:
        yield """<span>Cleanup complete</span>"""
    try:
        value = data['key']
    except KeyError::
        yield """<span>Not found</span>"""
        value = 'default'
    else:
        yield """<span>Success</span>"""
    finally:
        yield replace_markers(f"""<span>Done: ‹ESCAPE:{value}›</span>""")
