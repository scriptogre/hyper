from hyper import html, escape


@html
def Nested(
        *,
        title: str,
        items: list,
):
    # Nested components
    # <{Card}>
    def _card_default_slot():
        # <{CardHeader}>
        def _card_header_default_slot():
            yield f"""<h2>{escape(title)}</h2>"""
        yield from CardHeader(_card_header_default_slot())
        # </{CardHeader}>
        # <{CardBody}>
        def _card_body_default_slot():
            # <{List}>
            def _list_default_slot():
                for item in items:
                    # <{ListItem}>
                    def _list_item_default_slot():
                        yield f"""{escape(item)}"""
                    yield from ListItem(_list_item_default_slot())
                    # </{ListItem}>
            yield from List(_list_default_slot())
            # </{List}>
        yield from CardBody(_card_body_default_slot())
        # </{CardBody}>
    yield from Card(_card_default_slot())
    # </{Card}>

    # Component in control flow
    if title:
        # <{Alert}>
        def _alert_default_slot():
            yield f"""<span>{escape(title)}</span>"""
        yield from Alert(_alert_default_slot(), type_="info")
        # </{Alert}>
    # Components in loop
    for item in items:
        # <{Badge}>
        def _badge_default_slot():
            yield f"""{escape(item)}"""
        yield from Badge(_badge_default_slot(), color="blue")
        # </{Badge}>
