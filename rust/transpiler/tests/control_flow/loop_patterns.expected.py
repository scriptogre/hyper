from hyper import component, replace_markers, escape


@component
def LoopPatterns(*, items: list, pairs: list, names: list, scores: list, matrix: list):
    yield "<ul>"
    for i, item in enumerate(items):
        yield replace_markers(f"""<li data-index="{i}">‹ESCAPE:{item}›</li>""")
    yield "</ul>"
    yield "<ol>"
    for num, item in enumerate(items, start=1):
        yield replace_markers(f"""<li value="{num}">‹ESCAPE:{item}›</li>""")
    yield "</ol>"
    yield "<dl>"
    for key, value in pairs:
        yield replace_markers(f"""<dt>‹ESCAPE:{key}›</dt><dd>‹ESCAPE:{value}›</dd>""")
    yield "</dl>"
    yield "<table>"
    for name, score in zip(names, scores):
        yield replace_markers(f"""<tr><td>‹ESCAPE:{name}›</td><td>‹ESCAPE:{score}›</td></tr>""")
    yield "</table>"
    yield "<table>"
    for row in matrix:
        yield "<tr>"
        for cell in row:
            yield replace_markers(f"""<td>‹ESCAPE:{cell}›</td>""")
        yield "</tr>"
    yield "</table>"
    yield "<dl>"
    for k, v in items.items():
        yield replace_markers(f"""<dt>‹ESCAPE:{k}›</dt><dd>‹ESCAPE:{v}›</dd>""")
    yield "</dl>"
    yield "<ul>"
    for i in range(5):
        yield replace_markers(f"""<li>Item ‹ESCAPE:{i}›</li>""")
    yield "</ul>"
    yield "<ul>"
    for item in reversed(items):
        yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
    yield "</ul>"
