from hyper import html, replace_markers


@html
def Comprehensive(*, name: str, count: int = 0, is_active: bool = True, items: list = []):
    # Frontmatter comment

    # Body comment

    yield "<div class=\"container\">"

    # Indented comment
    yield replace_markers(f"""\
<span>Plain text</span>
    <p>‹ESCAPE:{name}›</p>
    <p>Count: ‹ESCAPE:{count + 1}›</p>
    """)
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
        yield replace_markers(f"""\
<li>‹ESCAPE:{item}›</li>
    """)

    yield "</div>"

