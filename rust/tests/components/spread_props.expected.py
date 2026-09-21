from hyper import component


@component
def SpreadProps(
        *,
        props: dict,
        label: str,
):
    yield from Button(**props).render(stream=True)
    yield from Button(label=label, **props).render(stream=True)
    yield from Button(disabled=True, **props).render(stream=True)
