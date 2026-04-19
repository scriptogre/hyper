from hyper import html, escape


@html
def BreakContinue(*, items: list, limit: int):

    # Break in for loop

    yield """<ul>"""

    for item in items:

        if item == "stop":

            break

        yield f"""\
<li>{escape(item)}</li>
    """

    yield """</ul>"""

    # Continue in for loop

    yield """<ul>"""

    for item in items:

        if item.startswith("_"):

            continue

        yield f"""\
<li>{escape(item)}</li>
    """

    yield """</ul>"""

    # Break in while loop

    count = 0

    while True:

        if count >= limit:

            break

        yield f"""\
<span>{escape(count)}</span>
    """
        count = count + 1


    # Nested break

    for outer in items:

        for inner in items:

            if inner == outer:

                break

            yield f"""\
<span>{escape(outer)}-{escape(inner)}</span>
    """


