from typing import Any
from hyper import html, escape, replace_markers


@html
def Kwargs(*, label: str, **attrs: Any):
    yield replace_markers(f"""<button attrs=‹SPREAD:{attrs}›>{escape(label)}</button>""")
