from hyper import html, render_attr, spread_attrs


@html
def ShorthandSpread(*, name: str, value: str, disabled: bool, props: dict):

    # Shorthand attributes
    yield f"""<input{spread_attrs(name)}{spread_attrs(value)}{render_attr("disabled", disabled)} />"""

    # Spread attributes
    yield f"""<div{spread_attrs(props)}>Content</div>"""

    # Mixed shorthand and spread
    yield f"""<button{spread_attrs(name)}{spread_attrs(props)}{render_attr("disabled", disabled)}>Click</button>"""

    # Spread with other attributes
    yield f"""<input type="text"{spread_attrs(props)} class="input" />"""
