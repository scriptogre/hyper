from typing import Any
from hyper import html, escape, spread_attrs


@html
def Kwargs(*, label: str, **attrs: Any):
    yield f"""<button{spread_attrs(attrs)}>{escape(label)}</button>"""
