from hyper import component, replace_markers


@component
def Comments(*, name: str, color: str):
    # Full-line comment (stripped)
    yield """<div>Content</div>"""

    # Another full-line comment
    yield """<span>Text</span>"""  # trailing comment after close tag

    # Hash in table header (not a comment)
    yield """<th scope="col">#</th>"""

    # Hash in URLs (not a comment)
    yield """\
<a href="#section">Jump</a>
<a href="/page#anchor">Link</a>"""

    # Hash in CSS colors (not a comment)
    yield """\
<div style="color: #ff0000">Red</div>
<div style="background: #fff">White</div>"""

    # HTML entities with hash
    yield """\
<span>&#35;</span>
<span>&#x23;</span>"""

    # Multiple hashes as content
    yield """\
<div>###</div>
<th>##</th>"""

    # Trailing comment after expression
    yield replace_markers(f"""<span>‹ESCAPE:{name}›</span>""")  # show name

    # Hash in expression (not a comment)
    yield replace_markers(f"""\
<span>‹ESCAPE:{name or '#'}›</span>
<span>‹ESCAPE:{"#" + name}›</span>""")

    # Hash inside paragraph content (not a comment)
    yield """<p>Text # with hash # multiple # times</p>"""

    # No space before hash (not a comment)
    yield """<span>Item</span># kept as content"""

    # Hash in attribute value (not a comment)
    yield """<div data-info="use # for comments">Info</div>"""

    # Trailing comment after self-closing tag
    yield """<br />"""  # self-closing

    # Hash in value attr + trailing comment
    yield """<input value="#" />"""  # input comment
