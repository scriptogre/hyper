from hyper import component, escape


@component
def NestedUnclosedFor():
    yield """<div>"""
    if show:
        for item in items:
            yield f"""<span>{escape(item)}</span>"""
        # Missing end for for loop
    yield """</div>"""
