from hyper import component, escape


@component
def UnclosedWhile():
    while count < 10:
        yield f"""<span>{escape(count)}</span>"""
