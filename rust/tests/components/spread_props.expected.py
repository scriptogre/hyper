from hyperhtml import component


@component
def SpreadProps(
        *,
        props: dict,
        label: str,
):
    yield from Button.stream(**props)
    yield from Button.stream(label=label, **props)
    yield from Button.stream(disabled=True, **props)
