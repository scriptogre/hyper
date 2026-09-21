from hyper import component, escape


@component
def UnclosedWith():
    with open("file.txt") as f:
        yield f"""<pre>{escape(f.read())}</pre>"""
