from hyper import component, replace_markers, escape


@component
def ComplexNested(*, sections: list, show_all: bool, user: dict):
    yield "<div class=\"container\">"
    if show_all:
        for section in sections:
            yield "<section id=\"section-{section['id']}\">"
            if section.get('visible', True):
                match section['type']:
                    case "header":
                        yield replace_markers(f"""<h1>‹ESCAPE:{section['title']}›</h1>""")
                    case "list":
                        yield "<ul>"
                        for item in section['items']:
                            if item.get('active'):
                                yield replace_markers(f"""<li class="active">‹ESCAPE:{item['name']}›</li>""")
                            else:
                                yield replace_markers(f"""<li>‹ESCAPE:{item['name']}›</li>""")
                        yield "</ul>"
                    case "text":
                        yield replace_markers(f"""<p>‹ESCAPE:{section['content']}›</p>""")
                    case _:
                        yield """<div>Unknown type</div>"""
            else:
                yield """<div class="hidden">Section hidden</div>"""
            yield "</section>"
    else:
        yield """<p>Content hidden</p>"""
    yield "</div>"
