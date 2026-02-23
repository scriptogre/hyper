from typing import Any
from hyper import html, escape, spread_attrs


@html
def KwargsCollector(*, title: str, **attrs: Any):
    yield f"""\
<div class="card"{spread_attrs(attrs)}>
    <h1>{escape(title)}</h1>
</div>"""
