from hyper import component, replace_markers, escape


@component
def Complex(*, data: dict, items: list, count: int):
    yield replace_markers(f"""<span>‹ESCAPE:{data['key']}›</span><span>‹ESCAPE:{data['nested']['deep']}›</span><span>‹ESCAPE:{data.get('key', 'default')}›</span><span>‹ESCAPE:{data['name'].strip().upper()}›</span><span>‹ESCAPE:{', '.join(items)}›</span><span>‹ESCAPE:{'yes' if count > 0 else 'no'}›</span><span>‹ESCAPE:{data['value'] if data.get('value') else 'N/A'}›</span><span>‹ESCAPE:{count * 2 + 1}›</span><span>‹ESCAPE:{count / 2}›</span><span>‹ESCAPE:{count ** 2}›</span><span>‹ESCAPE:{count % 3}›</span><span>‹ESCAPE:{count > 0 and count < 100}›</span><span>‹ESCAPE:{count >= 10 or count <= 5}›</span><span>‹ESCAPE:{items[0]}›</span><span>‹ESCAPE:{items[-1]}›</span><span>‹ESCAPE:{items[1:3]}›</span><span>‹ESCAPE:{items[::-1]}›</span><span>‹ESCAPE:{len(items)}›</span><span>‹ESCAPE:{count:03d}›</span><span>‹ESCAPE:{3.14159:.2f}›</span><span>‹ESCAPE:{data['name']:>20}›</span>""")
