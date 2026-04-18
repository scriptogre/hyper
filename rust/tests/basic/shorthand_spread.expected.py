from hyper import html, render_attr


@html
def ShorthandSpread(*, name: str, value: str, disabled: bool, props: dict):

    # Shorthand attributes
    yield f"""<input{render_attr("name", name)}{render_attr("value", value)}{render_attr("disabled", disabled)} />"""

    # Spread attributes
    yield f"""<div{render_attr("props", props)}>Content</div>"""

    # Mixed shorthand and spread
    yield f"""<button{render_attr("name", name)}{render_attr("props", props)}{render_attr("disabled", disabled)}>Click</button>"""

    # Spread with other attributes
    yield f"""<input type="text"{render_attr("props", props)} class="input" />"""
