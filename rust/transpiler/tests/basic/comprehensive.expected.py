from hyper import html, escape


@html
def Comprehensive(*, name: str, count: int = 0, is_active: bool = True, items: list = []):
    # Frontmatter comment

    # Body comment

    yield "<div class=\"container\">"

    # Indented comment
    yield f"""\
<span>Plain text</span>
    <p>{escape(name)}</p>
    <p>Count: {escape(count + 1)}</p>
    """
    if is_active:
        yield """<span>Active</span>"""
        # Trailing comment

    elif count > 0:
        yield """\
<span>Has count</span>
    """
    else:
        yield """\
<span>Inactive</span>
    """

    for item in items:
        yield f"""\
<li>{escape(item)}</li>
    """

    yield "</div>"

