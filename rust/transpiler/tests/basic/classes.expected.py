from hyper import component, replace_markers, escape


@component
def Classes(*, title: str, items: list):
    class Card:
        def __init__(self, title: str):
            self.title = title
        def render(self):
            yield replace_markers(f"""<div class="card"><h3>‹ESCAPE:{self.title}›</h3></div>""")
    class List:
        def __init__(self, items: list):
            self.items = items
        def render_item(self, item: str):
            yield replace_markers(f"""<li>‹ESCAPE:{item}›</li>""")
        def render(self):
            yield "<ul>"
            for item in self.items:
                self.render_item(item)
            yield "</ul>"
    card = Card(title)
    card.render()
