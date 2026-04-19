from hyper import html, escape


@html
async def Async(*, stream: object, connection: object, user_id: int):
    result = await fetch_user(user_id)
    yield f"""\
<h1>{escape(result.name)}</h1>
<p>{escape(await get_bio(user_id))}</p>"""
    async for item in stream:
        yield f"""<div>{escape(item)}</div>"""
    async with connection as conn:
        yield f"""<span>Connected: {escape(conn.id)}</span>"""
    async with connection as conn:
        async for message in conn.messages:
            yield f"""\
<p>{escape(message)}</p>
    """
