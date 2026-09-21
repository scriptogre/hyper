from hyper import component


@component
def EdgeCases(
        *,
        module: object,
        components: dict,
):
    # Component from module
    yield from module.Button(label="Click").render(stream=True)

    # Component from dict
    # <{components['Card']}>
    def _components_card_content():
        yield """<p>Content</p>"""
    yield from components['Card'](content=_components_card_content()).render(stream=True)
    # </{components['Card']}>

    # Empty component (not self-closing)
    yield from Wrapper().render(stream=True)

    # Component with only whitespace
    # <{Container}>
    def _container_content():
        pass
    yield from Container(content=_container_content()).render(stream=True)
    # </{Container}>

    # Deeply nested components
    # <{Outer}>
    def _outer_content():
        # <{Middle}>
        def _middle_content():
            # <{Inner}>
            def _inner_content():
                yield """<span>Deep</span>"""
            yield from Inner(content=_inner_content()).render(stream=True)
            # </{Inner}>
        yield from Middle(content=_middle_content()).render(stream=True)
        # </{Middle}>
    yield from Outer(content=_outer_content()).render(stream=True)
    # </{Outer}>
