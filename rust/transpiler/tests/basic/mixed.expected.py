from hyper import component, replace_markers


@component
def Mixed(*, id: str, theme: str, is_active: bool, count: int):
    # Static and dynamic mixed
    yield replace_markers(f"""<div class="card ‹ESCAPE:{theme}›" id="card-‹ESCAPE:{id}›">Content</div>""")

    # Multiple dynamic parts
    yield replace_markers(f"""<div class="‹ESCAPE:{theme}› ‹ESCAPE:{'active' if is_active else 'inactive'}› size-‹ESCAPE:{count}›">Content</div>""")

    # Quotes inside expressions
    yield replace_markers(f"""<div class="base ‹ESCAPE:{'special' if is_active else 'normal'}›">Content</div>""")

    # Data attributes
    yield replace_markers(f"""<div data-id="‹ESCAPE:{id}›" data-count="‹ESCAPE:{count}›" data-active="‹ESCAPE:{is_active}›">Content</div>""")

    # Aria attributes
    yield replace_markers(f"""<div aria-label="Item ‹ESCAPE:{id}›" aria-hidden="‹ESCAPE:{not is_active}›">Content</div>""")

    # Event handlers (Alpine.js style)
    yield replace_markers(f"""\
<button @click="handleClick(‹ESCAPE:{id}›)">Click</button>
<button x-on:click="toggle()">Toggle</button>""")
