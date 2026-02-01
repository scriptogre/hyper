from hyper import component, replace_markers


@component
def AttrEdgeCases(*, value: str):
    # Empty attribute value
    yield """\
<input value="" />
<div class="">Empty class</div>"""

    # Multiple classes (space-separated)
    yield """<div class="one two three four">Multiple classes</div>"""

    # Attribute with special characters
    yield """<div data-value="a&lt;b">Encoded</div>"""

    # XML namespace attributes
    yield """\
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <use xlink:href="#icon" />
</svg>"""

    # Boolean attributes (true = present, false = absent)
    yield """\
<input type="checkbox" checked />
<input type="checkbox" disabled />
<button disabled>Disabled</button>"""

    # Very long attribute
    yield """<div data-config="this is a very long attribute value that goes on and on and contains lots of text to test how the transpiler handles long attribute values in the output">Long</div>"""

    # Attribute with expression containing quotes
    yield replace_markers(f"""\
<div title="‹ESCAPE:{value}› said 'hello'">Quotes</div>
<div data-msg="‹ESCAPE:{value}› said &quot;hi&quot;">Double quotes</div>""")

    # Single vs double quotes
    yield """\
<div class="single-quoted">Single</div>
<div class="double-quoted">Double</div>"""
