from hyper import component


@component
def EmptyBlocks(*, flag: bool, items: list):
    # Empty if block (should generate pass)
    if flag:
        pass

    # Empty else block
    if flag:
        yield """<span>Yes</span>"""
    else:
        pass

    # Empty for block
    for item in items:
        pass

    # Empty while block
    while flag:
        pass

    # Empty match cases
    match flag:
        case True:
            pass
        case False:
            yield """<span>False</span>"""

    # Empty try blocks
    try:
        pass
    except:
        yield """<span>Error</span>"""

    # Empty function
    def empty():
        pass
