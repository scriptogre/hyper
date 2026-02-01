from typing import Any
from hyper import component, replace_markers


@component
def KwargsCollector(*, title: str, **attrs: Any):
    yield replace_markers(f"""\
<div class="card" attrs=‹SPREAD:{attrs}›>
    <h1>‹ESCAPE:{title}›</h1>
</div>""")
