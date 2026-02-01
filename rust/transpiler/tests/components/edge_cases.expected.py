from hyper import component


@component
def EdgeCases(*, module: object, components: dict):
    # Component from module
    yield from module.Button(label="Click")

    # Component from dict
    # <{components['Card']}>
    def _card():
        yield """<p>Content</p>"""
    yield from components['Card'](_card())
    # </{components['Card']}>

    # Empty component (not self-closing)
    yield from Wrapper()

    # Component with only whitespace
    # <{Container}>
    def _container():
        yield """
"""
    yield from Container(_container())
    # </{Container}>

    # Deeply nested components
    # <{Outer}>
    def _outer():
        # <{Middle}>
        def _middle():
            # <{Inner}>
            def _inner():
                yield """<span>Deep</span>"""
            yield from Inner(_inner())
            # </{Inner}>
        yield from Middle(_middle())
        # </{Middle}>
    yield from Outer(_outer())
    # </{Outer}>
