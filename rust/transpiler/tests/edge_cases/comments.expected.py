from hyper import escape, replace_markers

def Comments(name: str, color: str) -> str:
    _parts = []

    # Full-line comment (stripped)
    _parts.append("""<div>Content</div>""")

    # Another full-line comment
    _parts.append("""<span>Text</span>""")  # trailing comment after close tag

    # Hash in table header (not a comment)
    _parts.append("""<th scope="col">#</th>""")

    # Hash in URLs (not a comment)
    _parts.append("""<a href="#section">Jump</a>
<a href="/page#anchor">Link</a>""")

    # Hash in CSS colors (not a comment)
    _parts.append("""<div style="color: #ff0000">Red</div>
<div style="background: #fff">White</div>""")

    # HTML entities with hash
    _parts.append("""<span>&#35;</span>
<span>&#x23;</span>""")

    # Multiple hashes as content
    _parts.append("""<div>###</div>
<th>##</th>""")

    # Trailing comment after expression
    _parts.append(f"""<span>‹ESCAPE:{name}›</span>""")  # show name

    # Hash in expression (not a comment)
    _parts.append(f"""<span>‹ESCAPE:{name or '#'}›</span>
<span>‹ESCAPE:{"#" + name}›</span>""")

    # Hash inside paragraph content (not a comment)
    _parts.append("""<p>Text # with hash # multiple # times</p>""")

    # No space before hash (not a comment)
    _parts.append("""<span>Item</span># kept as content""")

    # Hash in attribute value (not a comment)
    _parts.append("""<div data-info="use # for comments">Info</div>""")

    # Trailing comment after self-closing tag
    _parts.append("""<br />""")  # self-closing

    # Hash in value attr + trailing comment
    _parts.append("""<input value="#" />""")  # input comment
    return replace_markers("".join(_parts))
