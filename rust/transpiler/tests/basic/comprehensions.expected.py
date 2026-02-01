from hyper import escape, replace_markers

def Comprehensions(items: list, data: dict) -> str:
    _parts = []
    _parts.append(f"""<span>‹ESCAPE:{[x * 2 for x in range(5)]}›</span><span>‹ESCAPE:{[item.upper() for item in items if item]}›</span><span>‹ESCAPE:{[x for x in items if x.startswith('a')]}›</span><span>{k: v.upper() for k, v in data.items()}</span><span>{k: v for k, v in data.items() if v}</span><span>{x for x in items}</span><span>‹ESCAPE:{sum(x for x in range(10))}›</span><span>‹ESCAPE:{','.join(str(x) for x in items)}›</span>""")
    return replace_markers("".join(_parts))
