from hyper import html, escape


@html
def Examples(*, user: dict, items: list, count: int = 0, is_active: bool = True):

    # Simple if
    if is_active:
        yield f"""<span>Active {escape(count)}</span>"""  # Comment
