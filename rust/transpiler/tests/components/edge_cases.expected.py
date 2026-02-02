from hyper import component


@component
def EdgeCases(*, module: object, components: dict):
    yield from module.Button(label="Click")
    # <{components['Card']}>
    def _components['_card']():
        yield """<p>Content</p>"""
    yield from components['Card'](_components['_card']())
    # </{components['Card']}>
    yield from Wrapper()
    yield from Container()
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
