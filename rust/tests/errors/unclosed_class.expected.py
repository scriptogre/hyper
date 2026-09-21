from hyper import component


@component
def UnclosedClass():
    class Card:
        def render(self):
            yield """<div>Card</div>"""
