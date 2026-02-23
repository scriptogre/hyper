from hyper import html, escape


@html
def Classes(*, title: str, items: list):

    # Simple class

    class Card:

        def __init__(self, title: str):

            self.title = title


        def render(self):
            yield f"""\
<div class="card">
            <h3>{escape(self.title)}</h3>
        </div>
    """


    # Class with multiple methods

    class List:

        def __init__(self, items: list):

            self.items = items


        def render_item(self, item: str):
            yield f"""\
<li>{escape(item)}</li>
    """

        def render(self):

            yield "<ul>"

            for item in self.items:

                self.render_item(item)


            yield "</ul>"



    # Instantiate and use

    card = Card(title)

    card.render()

