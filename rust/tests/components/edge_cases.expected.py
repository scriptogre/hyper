from hyper import html


@html
def EdgeCases(*, module: object, components: dict):
    # Component from module
    yield from module.Button(label="Click")

    # Component from dict
    # <{components['Card']}>
    def _components_card():
        yield """<p>Content</p>"""
    yield from components['Card'](_components_card())
    # </{components['Card']}>

    # Empty component (not self-closing)
    yield from Wrapper()

    # Component with only whitespace
    # <{Container}>
    def _container():
        pass
    yield from Container(_container())
    # </{Container}>

    # Deeply nested components
    # <{Outer}>
    def _outer():

        # <{Middle}>
        def _middle():

            # <{Inner}>
            def _inner():
                yield """\
<span>Deep</span>
        """
            yield from Inner(_inner())
            # </{Inner}>

        yield from Middle(_middle())
        # </{Middle}>

    yield from Outer(_outer())
    # </{Outer}>

