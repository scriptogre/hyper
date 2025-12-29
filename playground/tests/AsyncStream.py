async def AsyncStream(
        # Async iteration patterns
        stream: AsyncIterator[dict],
        async_resource: AsyncContextManager,
):
    _parts = []
    _parts.append(f"""<div class="async-patterns">""")
    # async for loop
    async for event in stream:
        _parts.append(f"""        <div class="event">
            <span class="type">{event['type']}</span>
            <span class="data">{event['data']}</span>
        </div>""")

    # async with
    async with async_resource as resource:
        _parts.append(f"""        <div class="resource">
            <span>Status: {resource.status}</span>
        </div>""")
    _parts.append(f"""</div>""")
    return "".join(_parts)
