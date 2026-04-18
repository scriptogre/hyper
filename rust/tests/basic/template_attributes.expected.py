from hyper import html, escape


@html
def TemplateAttributes(*, variant: str = "primary", size: str = "md", id: int = 0):

    # Template string in class attribute
    yield f"""<button class="btn btn-{escape(variant)}">Click</button>"""

    # Multiple expressions in one template attribute
    yield f"""<div data-info="{escape(id)}-{escape(variant)}">Info</div>"""

    # Template with size
    yield f"""<input class="input input-{escape(size)}" type="text" />"""

    # Mixed static and template
    yield f"""<a href="/users/{escape(id)}" class="link link-{escape(variant)}">Profile</a>"""

    # Adjacent expressions (no static text between)
    yield f"""<span data-key="{escape(id)}{escape(variant)}">Adjacent</span>"""

    # Expression at start and end of value
    yield f"""\
<div title="{escape(variant)} button">Start</div>
<div title="button {escape(variant)}">End</div>"""

    # Single expression is entire value
    yield f"""<div title="{escape(variant)}">Full</div>"""

    # Multiple attributes, some template some not
    yield f"""<a href="/items/{escape(id)}" class="link" data-label="item-{escape(variant)}">Multi</a>"""
