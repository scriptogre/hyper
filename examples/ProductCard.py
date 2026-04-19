from hyper import html, escape


@html
def ProductCard(*, name: str, price: float, image: str, on_sale: bool = False):

    yield f"<div class=\"{escape(["card", {"sale": on_sale}])}\">"
    yield f"""\
<img src="{escape(image)}" alt="{escape(name)}" />
    <h3>{escape(name)}</h3>
    """
    if on_sale:
        yield """\
<span class="badge">Sale</span>
    """
    yield f"""<p class="price">${price:.2f}</p>"""
    yield "</div>"

