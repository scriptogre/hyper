from hyper import component, replace_markers, escape


@component
async def AsyncLoops(*, stream: object, connection: object):
    async for item in stream:
        yield replace_markers(f"""<div>‹ESCAPE:{item}›</div>""")
    async with connection as conn:
        yield replace_markers(f"""<span>Connected: ‹ESCAPE:{conn.id}›</span>""")
    async with connection as conn:
        async for message in conn.messages:
            yield replace_markers(f"""<p>‹ESCAPE:{message}›</p>""")
