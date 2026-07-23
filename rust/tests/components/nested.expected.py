from hyperhtml import component, escape


@component
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
        yield from CardHeader.stream(content=_card_header_content())
        # </{CardHeader}>
        # <{CardBody}>
        def _card_body_content():
            # <{List}>
            def _list_content():
                for item in items:
                    # <{ListItem}>
                    def _list_item_content():
                        yield f"""{escape(item)}"""
                    yield from ListItem.stream(content=_list_item_content())
                    # </{ListItem}>
            yield from List.stream(content=_list_content())
            # </{List}>
        yield from CardBody.stream(content=_card_body_content())
        # </{CardBody}>
    yield from Card.stream(content=_card_content())
    # </{Card}>

    # Component in control flow
    if title:
        # <{Alert}>
        def _alert_content():
            yield f"""<span>{escape(title)}</span>"""
        yield from Alert.stream(content=_alert_content(), type="info")
        # </{Alert}>
    # Components in loop
    for item in items:
        # <{Badge}>
        def _badge_content():
            yield f"""{escape(item)}"""
        yield from Badge.stream(content=_badge_content(), color="blue")
        # </{Badge}>
