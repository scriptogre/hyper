from hyper import html, escape


@html
def MutableDefaults(
        *,
        name: str,
        items: list | None = None,
        config: dict | None = None,
        tags: set | None = None,
        plain_list: list = [],
        count: int | None = 0,
        label: str | None = None,
):
    if items is None:
        items = []
    if config is None:
        config = {"key": "value"}
    if tags is None:
        tags = set()
    yield f"""\
<div>{escape(name)}</div>
<span>{escape(items)}</span>
<span>{escape(config)}</span>"""
