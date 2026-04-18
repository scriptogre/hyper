from hyper import html, escape


@html
def Nested(*, title: str, items: list):

    # Nested components

    # <{Card}>
    def _card():

        # <{CardHeader}>
        def _card_header():
            yield f"""\
<h2>{escape(title)}</h2>
    """
        yield from CardHeader(_card_header())
        # </{CardHeader}>

        # <{CardBody}>
        def _card_body():

            # <{List}>
            def _list():

                for item in items:

                    # <{ListItem}>
                    def _list_item():
                        yield f"""\
{escape(item)}
                """
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
            yield f"""\
<span>{escape(title)}</span>
    """
        yield from Alert(_alert(), type="info")
        # </{Alert}>


    # Components in loop

    for item in items:

        # <{Badge}>
        def _badge():
            yield f"""\
{escape(item)}
    """
        yield from Badge(_badge(), color="blue")
        # </{Badge}>


