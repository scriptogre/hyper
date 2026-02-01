from hyper import component


@component
def Match(*, status: str):
    match status:
        case "loading":
            yield """<p>Loading...</p>"""
        case "error":
            yield """<p>Error!</p>"""
        case _:
            yield """<p>Ready</p>"""
