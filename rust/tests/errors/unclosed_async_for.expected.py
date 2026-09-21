from hyper import component, escape


@component
async def UnclosedAsyncFor(
        *,
        stream: object,
):
    async for item in stream:
        yield f"""<div>{escape(item)}</div>"""
