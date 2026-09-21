from hyper import component, escape


@component
async def UnclosedAsyncWith(
        *,
        conn: object,
):
    async with conn as c:
        yield f"""<div>{escape(c)}</div>"""
