from typing import Any
from hyper import html, escape, render_attr


@html
def Kwargs(*, label: str, **attrs: Any):
    yield f"""<button{render_attr("attrs", attrs)}>{escape(label)}</button>"""
