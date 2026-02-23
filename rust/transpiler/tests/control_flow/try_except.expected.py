from hyper import html, escape


@html
def TryExcept(*, data: dict, risky_func: object):

    # Simple try/except

    try:
        yield f"""<span>{escape(data['missing'])}</span>"""
    except KeyError:
        yield """<span>Key not found</span>"""

    # Try with multiple except

    try:
        yield f"""<span>{escape(risky_func())}</span>"""
    except ValueError as e:
        yield f"""<span>Value error: {escape(e)}</span>"""
    except TypeError as e:
        yield f"""<span>Type error: {escape(e)}</span>"""
    except Exception as e:
        yield f"""<span>Unknown error: {escape(e)}</span>"""

    # Try/except/else

    try:

        result = data['key']

    except KeyError:
        yield """<span>Missing</span>"""
    else:
        yield f"""<span>Found: {escape(result)}</span>"""

    # Try/except/finally

    try:
        yield f"""<span>{escape(data['value'])}</span>"""
    except:
        yield """<span>Error</span>"""
    finally:
        yield """<span>Cleanup complete</span>"""

    # Full try/except/else/finally

    try:

        value = data['key']

    except KeyError:
        yield """\
<span>Not found</span>
    """
        value = 'default'

    else:
        yield """<span>Success</span>"""
    finally:
        yield f"""<span>Done: {escape(value)}</span>"""

