from hyper import html, escape


@html
async def AwaitExpressions(*, user_id: int):

    result = await fetch_user(user_id)
    yield f"""\
<div>
    <h1>{escape(result.name)}</h1>
    <p>{escape(await get_bio(user_id))}</p>
</div>"""
