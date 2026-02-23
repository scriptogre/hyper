from hyper import html, escape


@html
def EscapedBraces(*, name: str):

    # Escaped braces for literal output
    yield """\
<p>Use {variable} for templates</p>
<p>JSON: {"key": "value"}</p>"""

    # Mixed escaped and dynamic
    yield f"""<p>Static {{braces}} and dynamic {escape(name)}</p>"""

    # In style blocks (CSS needs escaped braces)
    yield """\
<style>
    .card { background: white; }
    .card:hover { transform: scale(1.05); }
</style>"""

    # In script blocks (JS needs escaped braces)
    yield """\
<script>
    const obj = { name: "{name}" };
</script>"""

    # Alpine.js x-data
    yield """\
<div x-data="{{ open: false }}">
    <button @click="open = !open">Toggle</button>
</div>"""

    # Multiple consecutive
    yield """<p>{{nested}}</p>"""
