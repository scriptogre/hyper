from hyper import html, escape


@html
def Unicode(*, emoji: str, name: str):

    # Emoji in content
    yield """\
<span>Hello 👋</span>
<span>Stars: ⭐⭐⭐</span>"""

    # Emoji in expressions
    yield f"""\
<span>{escape(emoji)}</span>
<span>{escape('⭐' * 5)}</span>"""

    # CJK characters
    yield """\
<p lang="zh">你好世界</p>
<p lang="ja">こんにちは</p>
<p lang="ko">안녕하세요</p>"""

    # RTL text
    yield """\
<p dir="rtl" lang="ar">مرحبا بالعالم</p>
<p dir="rtl" lang="he">שלום עולם</p>"""

    # Mixed scripts
    yield f"""<p>{escape(name)}: 你好, مرحبا, שלום</p>"""

    # Special Unicode
    yield """\
<span>En dash: –</span>
<span>Em dash: —</span>
<span>Ellipsis: …</span>
<span>Quotes: "quoted"</span>
<span>Arrows: ← → ↑ ↓</span>
<span>Math: ∞ ≠ ≤ ≥ ± × ÷</span>"""
