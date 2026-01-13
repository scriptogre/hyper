from hyper import escape, replace_markers

def Classes(title: str, items: list) -> str:
    _parts = []
    class Card:
        def __init__(self, title: str):
            self.title = title
        def render(self):
            _parts.append(f"""<div class="card"><h3>‹ESCAPE:{self.title}›</h3></div>""")
    class List:
        def __init__(self, items: list):
            self.items = items
        def render_item(self, item: str):
            _parts.append(f"""<li>‹ESCAPE:{item}›</li>""")
        def render(self):
            _parts.append("<ul>")
            for item in self.items:
                self.render_item(item)
            _parts.append("</ul>")
    card = Card(title)
    card.render()
    return replace_markers("".join(_parts))
