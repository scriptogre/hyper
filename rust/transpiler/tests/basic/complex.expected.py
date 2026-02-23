from hyper import html, escape


@html
def Complex(*, data: dict, items: list, count: int):

    # Dict access
    yield f"""\
<span>{escape(data['key'])}</span>
<span>{escape(data['nested']['deep'])}</span>
<span>{escape(data.get('key', 'default'))}</span>"""

    # Method chaining
    yield f"""\
<span>{escape(data['name'].strip().upper())}</span>
<span>{escape(', '.join(items))}</span>"""

    # Ternary expressions
    yield f"""\
<span>{escape('yes' if count > 0 else 'no')}</span>
<span>{escape(data['value'] if data.get('value') else 'N/A')}</span>"""

    # Arithmetic
    yield f"""\
<span>{escape(count * 2 + 1)}</span>
<span>{escape(count / 2)}</span>
<span>{escape(count ** 2)}</span>
<span>{escape(count % 3)}</span>"""

    # Comparisons
    yield f"""\
<span>{escape(count > 0 and count < 100)}</span>
<span>{escape(count >= 10 or count <= 5)}</span>"""

    # List operations
    yield f"""\
<span>{escape(items[0])}</span>
<span>{escape(items[-1])}</span>
<span>{escape(items[1:3])}</span>
<span>{escape(items[::-1])}</span>
<span>{escape(len(items))}</span>"""

    # Format specifiers
    yield f"""\
<span>{count:03d}</span>
<span>{3.14159:.2f}</span>
<span>{data['name']:>20}</span>"""
