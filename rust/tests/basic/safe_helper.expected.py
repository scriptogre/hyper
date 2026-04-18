from hyper import html, escape, safe


@html
def SafeHelper(*, raw_html: str):
    yield f"""\
{escape(safe(raw_html))}
<div>{escape(safe("<b>bold</b>"))}</div>"""
