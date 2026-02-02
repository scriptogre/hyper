from hyper import component, replace_markers, escape


@component
def Kwargs(*, label: str, **attrs: Any):
    yield replace_markers(f"""<button attrs=‹SPREAD:{attrs}›>‹ESCAPE:{label}›</button>""")
