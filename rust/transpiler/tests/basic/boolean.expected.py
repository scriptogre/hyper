from hyper import html, replace_markers


@html
def Boolean(*, is_disabled: bool):
    yield replace_markers(f"""<button disabled=‹BOOL:{is_disabled}›>Submit</button>""")
