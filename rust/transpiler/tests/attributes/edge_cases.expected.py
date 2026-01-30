from hyper import replace_markers

def EdgeCases(value: str) -> str:
    _parts = []

    # Empty attribute value
    _parts.append("""<input value="" />
<div class="">Empty class</div>""")

    # Multiple classes (space-separated)
    _parts.append("""<div class="one two three four">Multiple classes</div>""")

    # Attribute with special characters
    _parts.append("""<div data-value="a&lt;b">Encoded</div>""")

    # XML namespace attributes
    _parts.append("""<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <use xlink:href="#icon" />
</svg>""")

    # Boolean attributes (true = present, false = absent)
    _parts.append("""<input type="checkbox" checked />
<input type="checkbox" disabled />
<button disabled>Disabled</button>""")

    # Very long attribute
    _parts.append("""<div data-config="this is a very long attribute value that goes on and on and contains lots of text to test how the transpiler handles long attribute values in the output">Long</div>""")

    # Attribute with expression containing quotes
    _parts.append(f"""<div title="‹ESCAPE:{value}› said 'hello'">Quotes</div>
<div data-msg='‹ESCAPE:{value}› said "hi"'>Double quotes</div>""")

    # Single vs double quotes
    _parts.append("""<div class='single-quoted'>Single</div>
<div class="double-quoted">Double</div>""")
    return replace_markers("".join(_parts))
