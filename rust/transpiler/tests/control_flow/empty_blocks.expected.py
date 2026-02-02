from hyper import component


@component
def EmptyBlocks(*, flag: bool, items: list):
    if flag:
        pass
    if flag:
        yield """<span>Yes</span>"""
    else:
        pass
    for item in items:
        pass
    while flag:
        pass
    match flag:
        case True:
            pass
        case False:
            yield """<span>False</span>"""
    try:
        pass
    except:
        yield """<span>Error</span>"""
    def empty():
        pass
