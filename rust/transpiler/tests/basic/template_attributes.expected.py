from hyper import html, replace_markers


@html
def TemplateAttributes(*, variant: str = "primary", size: str = "md", id: int = 0):

    # Template string in class attribute
    yield replace_markers(f"""<button class="btn btn-‹ESCAPE:{variant}›">Click</button>""")

    # Multiple expressions in one template attribute
    yield replace_markers(f"""<div data-info="‹ESCAPE:{id}›-‹ESCAPE:{variant}›">Info</div>""")

    # Template with size
    yield replace_markers(f"""<input class="input input-‹ESCAPE:{size}›" type="text" />""")

    # Mixed static and template
    yield replace_markers(f"""<a href="/users/‹ESCAPE:{id}›" class="link link-‹ESCAPE:{variant}›">Profile</a>""")
