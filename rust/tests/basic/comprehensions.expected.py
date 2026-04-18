from hyper import html, escape


@html
def Comprehensions(*, items: list, data: dict):

    # List comprehensions
    yield f"""\
<span>{escape([x * 2 for x in range(5)])}</span>
<span>{escape([item.upper() for item in items if item])}</span>
<span>{escape([x for x in items if x.startswith('a')])}</span>"""

    # Dict comprehensions (doubled braces for literal braces in f-string)
    yield """\
<span>{k: v.upper() for k, v in data.items()}</span>
<span>{k: v for k, v in data.items() if v}</span>"""

    # Set comprehensions
    yield """<span>{x for x in items}</span>"""

    # Generator expressions
    yield f"""\
<span>{escape(sum(x for x in range(10)))}</span>
<span>{escape(','.join(str(x) for x in items))}</span>"""
