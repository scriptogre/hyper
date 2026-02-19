from hyper import html, replace_markers


@html
def DataAria(*, is_hidden: bool):

    data = {"user-id": 123, "role": "admin"}

    aria = {"label": "Close", "hidden": is_hidden}
    yield replace_markers(f"""<div data=‹DATA:{data}› aria=‹ARIA:{aria}›>Content</div>""")
