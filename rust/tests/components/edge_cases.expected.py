from hyperhtml import component


@component
def EdgeCases(
        *,
        module: object,
        components: dict,
):
    # Component from module
    yield from module.Button.stream(label="Click")

    # Component from dict
    # <{components['Card']}>
    def _components_card_content():
        yield """<p>Content</p>"""
    yield from components['Card'].stream(content=_components_card_content())
    # </{components['Card']}>

    # Empty component (not self-closing)
    yield from Wrapper.stream()

    # Component with only whitespace
    # <{Container}>
    def _container_content():
        pass
    yield from Container.stream(content=_container_content())
    # </{Container}>

    # Deeply nested components
    # <{Outer}>
    def _outer_content():
        # <{Middle}>
        def _middle_content():
            # <{Inner}>
            def _inner_content():
                yield """<span>Deep</span>"""
            yield from Inner.stream(content=_inner_content())
            # </{Inner}>
        yield from Middle.stream(content=_middle_content())
        # </{Middle}>
    yield from Outer.stream(content=_outer_content())
    # </{Outer}>
