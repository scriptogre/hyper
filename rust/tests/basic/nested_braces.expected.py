from hyper import html, escape, render_class, render_style, render_data


@html
def NestedBraces(*, on_sale: bool, variant: str):

    # Dict in class attribute
    yield f"""\
<div class="{render_class(["card", {"sale": on_sale}])}">
    <span>Product</span>
</div>"""

    # Nested dict in style attribute
    yield f"""\
<div style="{render_style({"color": "red", "font": {"size": "12px"}})}">
    Styled
</div>"""

    # Set literal in expression
    yield f"""<span>{escape(frozenset({1, 2, 3}))}</span>"""

    # Dict comprehension in attribute
    yield f"""\
<div data="{escape("{k: v for k, v in items.items()}")}">
    Mapped
</div>"""
