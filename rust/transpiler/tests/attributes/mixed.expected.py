from hyper import escape, replace_markers

def Mixed(id: str, theme: str, is_active: bool, count: int) -> str:
    _parts = []

    # Static and dynamic mixed
    _parts.append(f"""<div class="card ‹ESCAPE:{theme}›" id="card-‹ESCAPE:{id}›">Content</div>""")

    # Multiple dynamic parts
    _parts.append(f"""<div class="‹ESCAPE:{theme}› ‹ESCAPE:{'active' if is_active else 'inactive'}› size-‹ESCAPE:{count}›">Content</div>""")

    # Quotes inside expressions
    _parts.append(f"""<div class="base ‹ESCAPE:{'special' if is_active else 'normal'}›">Content</div>""")

    # Data attributes
    _parts.append(f"""<div data-id="‹ESCAPE:{id}›" data-count="‹ESCAPE:{count}›" data-active="‹ESCAPE:{is_active}›">Content</div>""")

    # Aria attributes
    _parts.append(f"""<div aria-label="Item ‹ESCAPE:{id}›" aria-hidden="‹ESCAPE:{not is_active}›">Content</div>""")

    # Event handlers (Alpine.js style)
    _parts.append(f"""<button @click="handleClick(‹ESCAPE:{id}›)">Click</button>
<button x-on:click="toggle()">Toggle</button>""")
    return replace_markers("".join(_parts))
