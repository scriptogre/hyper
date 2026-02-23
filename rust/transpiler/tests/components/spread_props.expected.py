from hyper import html


@html
def SpreadProps(*, props: dict, label: str):

    yield from Button(**props)

    yield from Button(label=label, **props)

    yield from Button(disabled=True, **props)

