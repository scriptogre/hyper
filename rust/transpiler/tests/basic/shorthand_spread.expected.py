from hyper import component, replace_markers


@component
def ShorthandSpread(*, name: str, value: str, disabled: bool, props: dict):
    yield replace_markers(f"""<input name=‹SPREAD:{name}› value=‹SPREAD:{value}› disabled=‹BOOL:{disabled}› /><div props=‹SPREAD:{props}›>Content</div><button name=‹SPREAD:{name}› props=‹SPREAD:{props}› disabled=‹BOOL:{disabled}›>Click</button><input type="text" props=‹SPREAD:{props}› class="input" />""")
