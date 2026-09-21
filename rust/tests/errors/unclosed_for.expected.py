from hyper import component, escape


@component
def UnclosedFor():
    for item in items:
        yield f"""<li>{escape(item)}</li>"""
