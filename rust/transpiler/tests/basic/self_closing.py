def SelfClosing() -> str:
    _parts = []
    _parts.append("""<br /><hr /><img src="photo.jpg" alt="Photo" /><input type="text" name="field" /><div /><span />""")
    return "".join(_parts)
