from hyper import html, escape


@html
def LoopPatterns(*, items: list, pairs: list, names: list, scores: list, matrix: list):

    # Enumerate

    yield "<ul>"

    for i, item in enumerate(items):
        yield f"""\
<li data-index="{escape(i)}">{escape(item)}</li>
    """

    yield "</ul>"

    # Enumerate with start

    yield "<ol>"

    for num, item in enumerate(items, start=1):
        yield f"""\
<li value="{escape(num)}">{escape(item)}</li>
    """

    yield "</ol>"

    # Tuple unpacking

    yield "<dl>"

    for key, value in pairs:
        yield f"""\
<dt>{escape(key)}</dt>
        <dd>{escape(value)}</dd>
    """

    yield "</dl>"

    # Zip

    yield "<table>"

    for name, score in zip(names, scores):
        yield f"""\
<tr>
            <td>{escape(name)}</td>
            <td>{escape(score)}</td>
        </tr>
    """

    yield "</table>"

    # Nested loops

    yield "<table>"

    for row in matrix:

        yield "<tr>"

        for cell in row:
            yield f"""\
<td>{escape(cell)}</td>
            """

        yield "</tr>"


    yield "</table>"

    # Dict items

    yield "<dl>"

    for k, v in items.items():
        yield f"""\
<dt>{escape(k)}</dt>
        <dd>{escape(v)}</dd>
    """

    yield "</dl>"

    # Range

    yield "<ul>"

    for i in range(5):
        yield f"""\
<li>Item {escape(i)}</li>
    """

    yield "</ul>"

    # Reversed

    yield "<ul>"

    for item in reversed(items):
        yield f"""\
<li>{escape(item)}</li>
    """

    yield "</ul>"

