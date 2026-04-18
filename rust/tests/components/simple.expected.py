from hyper import html, escape


@html
def Simple(*, title: str, count: int):
    yield f"""\
<div>
    <h1>{escape(title)}</h1>
    <p>Count: {escape(count)}</p>
</div>"""
