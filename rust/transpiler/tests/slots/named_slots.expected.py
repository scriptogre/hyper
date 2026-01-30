def NamedSlots(*, _children: str = "", _sidebar_children: str = "") -> str:
    _parts = []
    _parts.append("""<div class="layout">
    <aside>""")
    _parts.append(_sidebar_children)
    _parts.append("""</aside>
    <main>""")
    _parts.append(_children)
    _parts.append("""</main>
</div>""")
    return "".join(_parts)
