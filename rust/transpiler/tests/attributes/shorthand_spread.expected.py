from hyper import replace_markers

def ShorthandSpread(name: str, value: str, disabled: bool, props: dict) -> str:
    _parts = []
    _parts.append(f"""<input name=‹SPREAD:{name}› value=‹SPREAD:{value}› disabled=‹BOOL:{disabled}› /><div ‹SPREAD:{props}›>Content</div><button name=‹SPREAD:{name}› ‹SPREAD:{props}› disabled=‹BOOL:{disabled}›>Click</button><input type="text" ‹SPREAD:{props}› class="input" />""")
    return replace_markers("".join(_parts))
