from typing import Any
from hyper import html, escape, replace_markers


@html
def KwargsCollector(*, title: str, **attrs: Any):
    yield replace_markers(f"""\
<div class="card" attrs=‹SPREAD:{attrs}›>
    <h1>{escape(title)}</h1>
</div>""")
