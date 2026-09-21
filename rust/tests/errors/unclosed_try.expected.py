from hyper import component, escape


@component
def UnclosedTry():
    try:
        yield f"""<span>{escape(risky())}</span>"""
    except ValueError:
        yield """<span>Error</span>"""
