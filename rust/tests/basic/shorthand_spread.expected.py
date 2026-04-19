from hyper import html, render_attr


@html
def ShorthandSpread(*, name: str, value: str, disabled: bool, title: str):

    # Shorthand attributes
    yield f"""<input{render_attr("name", name)}{render_attr("value", value)}{render_attr("disabled", disabled)} />"""

    # Multiple shorthand
    yield f"""<div{render_attr("title", title)}>Content</div>"""

    # Mixed shorthand
    yield f"""<button{render_attr("name", name)}{render_attr("title", title)}{render_attr("disabled", disabled)}>Click</button>"""

    # Shorthand with static attributes
    yield f"""<input type="text"{render_attr("name", name)} class="input" />"""
