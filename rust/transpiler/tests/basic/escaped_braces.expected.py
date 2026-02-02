from hyper import component, replace_markers, escape


@component
def EscapedBraces(*, name: str):
    yield replace_markers(f"""<p>Use {variable} for templates</p><p>JSON: {"key": "value"}</p><p>Static {braces} and dynamic ‹ESCAPE:{name}›</p><style>.card { background: white; }.card:hover { transform: scale(1.05); }</style><script>const obj = { name: "{name}" };</script><div x-data="{{ open: false }}"><button @click="open = !open">Toggle</button></div><p>{{nested}}</p>""")
