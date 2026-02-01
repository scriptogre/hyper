from hyper import component, replace_markers


@component
def EscapedBraces(*, name: str):
    # Escaped braces for literal output
    yield """\
<p>Use {variable} for templates</p>
<p>JSON: {"key": "value"}</p>"""

    # Mixed escaped and dynamic
    yield replace_markers(f"""<p>Static {{braces}} and dynamic ‹ESCAPE:{name}›</p>""")

    # In style blocks (CSS needs escaped braces)
    yield """\
<style>
    .card { background: white; }
    .card:hover { transform: scale(1.05); }
</style>"""

    # In script blocks (JS needs escaped braces)
    yield replace_markers(f"""\
<script>
    const obj = {{ name: "‹ESCAPE:{name}›" }};
</script>""")

    # Alpine.js x-data
    yield """\
<div x-data="{ open: false }">
    <button @click="open = !open">Toggle</button>
</div>"""

    # Multiple consecutive
    yield """<p>{{nested}}</p>"""
