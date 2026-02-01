from hyper import escape, replace_markers

def ComplexNested(sections: list, show_all: bool, user: dict) -> str:
    _parts = []
    _parts.append("<div class=\"container\">")
    if show_all:
        for section in sections:
            _parts.append("<section id=\"section-{section['id']}\">")
            if section.get('visible', True):
                match section['type']:
                    case "header":
                        _parts.append(f"""<h1>‹ESCAPE:{section['title']}›</h1>""")
                    case "list":
                        _parts.append("<ul>")
                        for item in section['items']:
                            if item.get('active'):
                                _parts.append(f"""<li class="active">‹ESCAPE:{item['name']}›</li>""")
                            else:
                                _parts.append(f"""<li>‹ESCAPE:{item['name']}›</li>""")
                        _parts.append("</ul>")
                    case "text":
                        _parts.append(f"""<p>‹ESCAPE:{section['content']}›</p>""")
                    case _:
                        _parts.append("""<div>Unknown type</div>""")
            else:
                _parts.append("""<div class="hidden">Section hidden</div>""")
            _parts.append("</section>")
    else:
        _parts.append("""<p>Content hidden</p>""")
    _parts.append("</div>")
    return replace_markers("".join(_parts))
