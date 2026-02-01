from hyper import component, replace_markers


@component
async def AsyncLoops(*, stream: object, connection: object):
    # Async for loop
    async for item in stream:
        yield replace_markers(f"""<div>‹ESCAPE:{item}›</div>""")

    # Async with statement
    async with connection as conn:
        yield replace_markers(f"""<span>Connected: ‹ESCAPE:{conn.id}›</span>""")

    # Nested async
    async with connection as conn:
        async for message in conn.messages:
            yield replace_markers(f"""<p>‹ESCAPE:{message}›</p>""")
