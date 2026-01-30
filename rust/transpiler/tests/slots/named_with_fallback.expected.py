def NamedWithFallback(title: str, *, _children: str = "", _footer_children: str = "", _header_children: str = "", _sidebar_children: str = "") -> str:
    _parts = []
    _parts.append("""<div class="layout">
    <header>
        """)
    _parts.append(_header_children or """<h1>Default Header</h1>""")
    _parts.append("""
    </header>

    <nav>
        """)
    _parts.append(_sidebar_children or """<p>Default sidebar content</p>""")
    _parts.append("""
    </nav>

    <main>
        """)
    _parts.append(_children)
    _parts.append("""
    </main>

    <footer>
        """)
    _parts.append(_footer_children or """<p>Default footer</p>""")
    _parts.append("""
    </footer>
</div>""")
    return "".join(_parts)
