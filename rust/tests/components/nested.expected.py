from hyper import html, escape


@html
def Nested(
        *,
        title: str,
        items: list,
):
    # Nested components
    # <{Card}>
    def _card_content():
        # <{CardHeader}>
        def _card_header_content():
            yield f"""<h2>{escape(title)}</h2>"""
        yield from CardHeader(_card_header_content())
        # </{CardHeader}>
        # <{CardBody}>
        def _card_body_content():
            # <{List}>
            def _list_content():
                for item in items:
                    # <{ListItem}>
                    def _list_item_content():
                        yield f"""{escape(item)}"""
                    yield from ListItem(_list_item_content())
                    # </{ListItem}>
            yield from List(_list_content())
            # </{List}>
        yield from CardBody(_card_body_content())
        # </{CardBody}>
    yield from Card(_card_content())
    # </{Card}>

    # Component in control flow
    if title:
        # <{Alert}>
        def _alert_content():
            yield f"""<span>{escape(title)}</span>"""
        yield from Alert(_alert_content(), type="info")
        # </{Alert}>
    # Components in loop
    for item in items:
        # <{Badge}>
        def _badge_content():
            yield f"""{escape(item)}"""
        yield from Badge(_badge_content(), color="blue")
        # </{Badge}>
