from typing import Any
from hyper import escape, replace_markers

def KwargsCollector(title: str, **attrs: Any) -> str:
    _parts = []
    _parts.append(f"""<div class="card" ‹SPREAD:{attrs}›>
    <h1>‹ESCAPE:{title}›</h1>
</div>""")
    return replace_markers("".join(_parts))
