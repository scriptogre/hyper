from hyper import html, escape


@html
def With():

    with open("file.txt") as f:
        yield f"""<pre>{escape(f.read())}</pre>"""

