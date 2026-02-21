from hyper import html, replace_markers


@html
async def AwaitExpressions(*, user_id: int):

    result = await fetch_user(user_id)
    yield replace_markers(f"""\
<div>
    <h1>‹ESCAPE:{result.name}›</h1>
    <p>‹ESCAPE:{await get_bio(user_id)}›</p>
</div>""")
