from hyper import html, escape


@html
async def AsyncLoops(*, stream: object, connection: object):

    # Async for loop

    async for item in stream:
        yield f"""<div>{escape(item)}</div>"""

    # Async with statement

    async with connection as conn:
        yield f"""<span>Connected: {escape(conn.id)}</span>"""

    # Nested async

    async with connection as conn:

        async for message in conn.messages:
            yield f"""\
<p>{escape(message)}</p>
    """


