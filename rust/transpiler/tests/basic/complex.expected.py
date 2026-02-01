from hyper import component, replace_markers


@component
def Complex(*, data: dict, items: list, count: int):
    # Dict access
    yield replace_markers(f"""\
<span>‹ESCAPE:{data['key']}›</span>
<span>‹ESCAPE:{data['nested']['deep']}›</span>
<span>‹ESCAPE:{data.get('key', 'default')}›</span>""")

    # Method chaining
    yield replace_markers(f"""\
<span>‹ESCAPE:{data['name'].strip().upper()}›</span>
<span>‹ESCAPE:{', '.join(items)}›</span>""")

    # Ternary expressions
    yield replace_markers(f"""\
<span>‹ESCAPE:{'yes' if count > 0 else 'no'}›</span>
<span>‹ESCAPE:{data['value'] if data.get('value') else 'N/A'}›</span>""")

    # Arithmetic
    yield replace_markers(f"""\
<span>‹ESCAPE:{count * 2 + 1}›</span>
<span>‹ESCAPE:{count / 2}›</span>
<span>‹ESCAPE:{count ** 2}›</span>
<span>‹ESCAPE:{count % 3}›</span>""")

    # Comparisons
    yield replace_markers(f"""\
<span>‹ESCAPE:{count > 0 and count < 100}›</span>
<span>‹ESCAPE:{count >= 10 or count <= 5}›</span>""")

    # List operations
    yield replace_markers(f"""\
<span>‹ESCAPE:{items[0]}›</span>
<span>‹ESCAPE:{items[-1]}›</span>
<span>‹ESCAPE:{items[1:3]}›</span>
<span>‹ESCAPE:{items[::-1]}›</span>
<span>‹ESCAPE:{len(items)}›</span>""")

    # Format specifiers
    yield replace_markers(f"""\
<span>‹ESCAPE:{count:03d}›</span>
<span>‹ESCAPE:{3.14159:.2f}›</span>
<span>‹ESCAPE:{data['name']:>20}›</span>""")
