from hyper import escape, replace_markers

def DefaultSlot(title: str, *, _children: str = "") -> str:
    _parts = []
    _parts.append(f"""<div class="card">
    <h2>‹ESCAPE:{title}›</h2>
    """)
    _parts.append(_children)
    _parts.append("""
</div>""")
    return replace_markers("".join(_parts))
