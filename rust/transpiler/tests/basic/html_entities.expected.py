from hyper import component, replace_markers


@component
def HtmlEntities():
    # Common entities
    yield """\
<p>&lt;script&gt; is escaped</p>
<p>Ampersand: &amp;</p>
<p>Non-breaking space: hello&nbsp;world</p>
<p>Copyright: &copy; 2024</p>
<p>Registered: &reg;</p>
<p>Trademark: &trade;</p>"""

    # Currency
    yield """\
<p>Dollar: &#36;100</p>
<p>Euro: &euro;50</p>
<p>Pound: &pound;30</p>
<p>Yen: &yen;1000</p>"""

    # Quotes
    yield """\
<p>&quot;quoted&quot;</p>
<p>&apos;single&apos;</p>
<p>&ldquo;smart quotes&rdquo;</p>"""

    # Numeric entities
    yield """\
<p>&#60;angle brackets&#62;</p>
<p>&#x3C;hex entities&#x3E;</p>"""

    # Mixed with expressions
    yield replace_markers(f"""<p>&copy; ‹ESCAPE:{2024}› All rights reserved</p>""")
