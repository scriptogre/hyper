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
