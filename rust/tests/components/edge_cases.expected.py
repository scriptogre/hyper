from hyper import html


@html
def EdgeCases(
        *,
        module: object,
        components: dict,
):
    # Component from module
    yield from module.Button(label="Click")

    # Component from dict
    # <{components['Card']}>
    def _components_card_default_slot():
        yield """<p>Content</p>"""
    yield from components['Card'](_components_card_default_slot())
    # </{components['Card']}>

    # Empty component (not self-closing)
    yield from Wrapper()

    # Component with only whitespace
    # <{Container}>
    def _container_default_slot():
        pass
    yield from Container(_container_default_slot())
    # </{Container}>

    # Deeply nested components
    # <{Outer}>
    def _outer_default_slot():
        # <{Middle}>
        def _middle_default_slot():
            # <{Inner}>
            def _inner_default_slot():
                yield """<span>Deep</span>"""
            yield from Inner(_inner_default_slot())
            # </{Inner}>
        yield from Middle(_middle_default_slot())
        # </{Middle}>
    yield from Outer(_outer_default_slot())
    # </{Outer}>
