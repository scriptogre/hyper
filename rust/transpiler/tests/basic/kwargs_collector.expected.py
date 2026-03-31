from typing import Any
from hyper import html, escape, render_attr


@html
def KwargsCollector(*, title: str, **attrs: Any):
    yield f"""\
<div class="card"{render_attr("attrs", attrs)}>
    <h1>{escape(title)}</h1>
</div>"""
