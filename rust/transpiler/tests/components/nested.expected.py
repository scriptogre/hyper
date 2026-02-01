from hyper import component, replace_markers


@component
def Nested(*, title: str, items: list):
    # Nested components
    # <{Card}>
    def _card():
        # <{CardHeader}>
        def _card_header():
            yield replace_markers(f"""<h2>‹ESCAPE:{title}›</h2>""")
        yield from CardHeader(_card_header())
        # </{CardHeader}>
        # <{CardBody}>
        def _card_body():
            # <{List}>
            def _list():
                for item in items:
                    # <{ListItem}>
                    def _list_item():
                        yield replace_markers(f"""‹ESCAPE:{item}›""")
                    yield from ListItem(_list_item())
                    # </{ListItem}>
            yield from List(_list())
            # </{List}>
        yield from CardBody(_card_body())
        # </{CardBody}>
    yield from Card(_card())
    # </{Card}>

    # Component in control flow
    if title:
        # <{Alert}>
        def _alert():
            yield replace_markers(f"""<span>‹ESCAPE:{title}›</span>""")
        yield from Alert(_alert(), type="info")
        # </{Alert}>

    # Components in loop
    for item in items:
        # <{Badge}>
        def _badge():
            yield replace_markers(f"""‹ESCAPE:{item}›""")
        yield from Badge(_badge(), color="blue")
        # </{Badge}>
