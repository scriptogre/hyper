from hyper import escape, replace_markers

def Comments(name: str, color: str) -> str:
    _parts = []
    _parts.append(f"""<div>Content</div><span>Text</span><th scope="col">#</th><a href="#section">Jump</a><a href="/page#anchor">Link</a><div style="color: #ff0000">Red</div><div style="background: #fff">White</div><span>&#35;</span><span>&#x23;</span><div>###</div><th>##</th><span>‹ESCAPE:{name}›</span><span>‹ESCAPE:{name or '#'}›</span><span>‹ESCAPE:{"#" + name}›</span><p>Text # with hash # multiple # times</p><span>Item</span># kept as content<div data-info="use # for comments">Info</div><br /><input value="#" />""")
    return replace_markers("".join(_parts))
