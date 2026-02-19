from hyper import html, replace_markers


@html
def Comprehensions(*, items: list, data: dict):

    # List comprehensions
    yield replace_markers(f"""\
<span>‹ESCAPE:{[x * 2 for x in range(5)]}›</span>
<span>‹ESCAPE:{[item.upper() for item in items if item]}›</span>
<span>‹ESCAPE:{[x for x in items if x.startswith('a')]}›</span>""")

    # Dict comprehensions (doubled braces for literal braces in f-string)
    yield """\
<span>{k: v.upper() for k, v in data.items()}</span>
<span>{k: v for k, v in data.items() if v}</span>"""

    # Set comprehensions
    yield """<span>{x for x in items}</span>"""

    # Generator expressions
    yield replace_markers(f"""\
<span>‹ESCAPE:{sum(x for x in range(10))}›</span>
<span>‹ESCAPE:{','.join(str(x) for x in items)}›</span>""")
