from hyper import component, escape


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
        yield from CardHeader(content=_card_header_content()).render(stream=True)
        # </{CardHeader}>
        # <{CardBody}>
        def _card_body_content():
            # <{List}>
            def _list_content():
                for item in items:
                    # <{ListItem}>
                    def _list_item_content():
                        yield f"""{escape(item)}"""
                    yield from ListItem(content=_list_item_content()).render(stream=True)
                    # </{ListItem}>
            yield from List(content=_list_content()).render(stream=True)
            # </{List}>
        yield from CardBody(content=_card_body_content()).render(stream=True)
        # </{CardBody}>
    yield from Card(content=_card_content()).render(stream=True)
    # </{Card}>

    # Component in control flow
    if title:
        # <{Alert}>
        def _alert_content():
            yield f"""<span>{escape(title)}</span>"""
        yield from Alert(content=_alert_content(), type="info").render(stream=True)
        # </{Alert}>
    # Components in loop
    for item in items:
        # <{Badge}>
        def _badge_content():
            yield f"""{escape(item)}"""
        yield from Badge(content=_badge_content(), color="blue").render(stream=True)
        # </{Badge}>
