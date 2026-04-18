from hyper import html, escape


@html
def Ternary(*, count: int):
    yield f"""<span>{escape(count)} {escape("item" if count == 1 else "items")}</span>"""
