from typing import Any, Callable
from hyper import html, replace_markers


@html
def ComplexTypes(*, simple: str, with_default: str = "default", optional: str | None = None, items: list[str], mapping: dict[str, int], nested: list[dict[str, Any]], callback: Callable[[int], str], **kwargs: Any):
    yield replace_markers(f"""\
<div>
    <span>‹ESCAPE:{simple}›</span>
    <span>‹ESCAPE:{with_default}›</span>
    <span>‹ESCAPE:{optional or 'none'}›</span>
    <span>‹ESCAPE:{len(items)}›</span>
    <span>‹ESCAPE:{len(mapping)}›</span>
</div>""")
