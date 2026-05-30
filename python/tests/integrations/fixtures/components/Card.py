from hyper import html, escape


@html
def Card(
        *,
        title: str,
        body: str,
):
    yield f"""\
<div class="card">
    <h2>{escape(title)}</h2>
    <p>{escape(body)}</p>
</div>"""
