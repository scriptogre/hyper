from hyper import component, replace_markers


@component
def ShorthandSpread(*, name: str, value: str, disabled: bool, props: dict):
    # Shorthand attributes
    yield replace_markers(f"""<input name=‹SPREAD:{name}› value=‹SPREAD:{value}› disabled=‹BOOL:{disabled}› />""")

    # Spread attributes
    yield replace_markers(f"""<div props=‹SPREAD:{props}›>Content</div>""")

    # Mixed shorthand and spread
    yield replace_markers(f"""<button name=‹SPREAD:{name}› props=‹SPREAD:{props}› disabled=‹BOOL:{disabled}›>Click</button>""")

    # Spread with other attributes
    yield replace_markers(f"""<input type="text" props=‹SPREAD:{props}› class="input" />""")
