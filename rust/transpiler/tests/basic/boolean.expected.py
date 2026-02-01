from hyper import component, replace_markers


@component
def Boolean(*, is_disabled: bool):
    yield replace_markers(f"""<button disabled=‹BOOL:{is_disabled}›>Submit</button>""")
