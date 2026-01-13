from hyper import escape
from typing import AsyncIterable
async def async_for(items: AsyncIterable[str]):
    _parts = []

    _parts.append(f"""<ul>""")
    async for item in items:
        _parts = []
        _parts.append(f"""<li>{escape(item)}</li>""")
        return ''.join(_parts)

    _parts.append(f"""</ul>""")
    return ''.join(_parts)
