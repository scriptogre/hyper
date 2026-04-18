from hyper import html, escape


@html
def Mixed(*, id: str, theme: str, is_active: bool, count: int):

    # Static and dynamic mixed
    yield f"""<div class="card {escape(theme)}" id="card-{escape(id)}">Content</div>"""

    # Multiple dynamic parts
    yield f"""<div class="{escape(theme)} {escape('active' if is_active else 'inactive')} size-{escape(count)}">Content</div>"""

    # Quotes inside expressions
    yield f"""<div class="base {escape('special' if is_active else 'normal')}">Content</div>"""

    # Data attributes
    yield f"""<div data-id="{escape(id)}" data-count="{escape(count)}" data-active="{escape(is_active)}">Content</div>"""

    # Aria attributes
    yield f"""<div aria-label="Item {escape(id)}" aria-hidden="{escape(not is_active)}">Content</div>"""

    # Event handlers (Alpine.js style)
    yield f"""\
<button @click="handleClick({escape(id)})">Click</button>
<button x-on:click="toggle()">Toggle</button>"""
