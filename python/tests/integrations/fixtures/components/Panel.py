from collections.abc import Iterable
from hyper import html, escape


@html
def Panel(
        _default_slot: Iterable[str] | None = None,
        *,
        title: str,
        _actions_slot: Iterable[str] | None = None,
):
    yield """<section class="panel">"""
    yield f"""<h2>{escape(title)}</h2>"""
    # <{...}>
    if _default_slot is not None:
        yield from _default_slot
    # </{...}>
    yield """<footer>"""
    # <{...actions}>
    if _actions_slot is not None:
        yield from _actions_slot
    else:
        yield """<span>No actions</span>"""
    # </{...actions}>
    yield """</footer>"""
    yield """</section>"""
