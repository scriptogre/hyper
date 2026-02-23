from typing import Any, Callable
from hyper import html, escape


@html
def ComplexTypes(*, simple: str, with_default: str = "default", optional: str | None = None, items: list[str], mapping: dict[str, int], nested: list[dict[str, Any]], callback: Callable[[int], str], **kwargs: Any):
    yield f"""\
<div>
    <span>{escape(simple)}</span>
    <span>{escape(with_default)}</span>
    <span>{escape(optional or 'none')}</span>
    <span>{escape(len(items))}</span>
    <span>{escape(len(mapping))}</span>
</div>"""
