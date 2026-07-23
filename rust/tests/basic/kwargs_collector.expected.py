from typing import Any
from hyperhtml import component, escape, spread_attrs


@component
def KwargsCollector(
        *,
        title: str,
        **attrs: Any,
):
    yield f"""\
<div class="card"{spread_attrs(attrs)}>
    <h1>{escape(title)}</h1>
</div>"""
