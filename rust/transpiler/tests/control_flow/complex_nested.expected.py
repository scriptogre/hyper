from hyper import html, escape


@html
def ComplexNested(*, sections: list, show_all: bool, user: dict):

    yield "<div class=\"container\">"

    if show_all:

        for section in sections:

            yield "<section id=\"section-{escape(section['id'])}\">"

            if section.get('visible', True):

                match section['type']:
                    case "header":
                        yield f"""\
<h1>{escape(section['title'])}</h1>
                        """
                    case "list":

                        yield "<ul>"

                        for item in section['items']:

                            if item.get('active'):
                                yield f"""\
<li class="active">{escape(item['name'])}</li>
                                    """
                            else:
                                yield f"""\
<li>{escape(item['name'])}</li>
                                    """


                        yield "</ul>"

                    case "text":
                        yield f"""\
<p>{escape(section['content'])}</p>
                        """
                    case _:
                        yield """\
<div>Unknown type</div>
                    """

            else:
                yield """\
<div class="hidden">Section hidden</div>
                """

            yield "</section>"


    else:
        yield """\
<p>Content hidden</p>
    """

    yield "</div>"

