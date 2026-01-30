from hyper import replace_markers

def ShorthandSpread(name: str, value: str, disabled: bool, props: dict) -> str:
    _parts = []

    # Shorthand attributes
    _parts.append(f"""<input name="‹ESCAPE:{name}›" value="‹ESCAPE:{value}›" disabled=‹BOOL:{disabled}› />""")

    # Spread attributes
    _parts.append(f"""<div ‹SPREAD:{props}›>Content</div>""")

    # Mixed shorthand and spread
    _parts.append(f"""<button name="‹ESCAPE:{name}›" ‹SPREAD:{props}› disabled=‹BOOL:{disabled}›>Click</button>""")

    # Spread with other attributes
    _parts.append(f"""<input type="text" ‹SPREAD:{props}› class="input" />""")
    return replace_markers("".join(_parts))
