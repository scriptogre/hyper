from hyper import component


@component
def UnclosedMatch():
    match status:
        case "active":
            yield """<span>Active</span>"""
        case _:
            yield """<span>Unknown</span>"""
