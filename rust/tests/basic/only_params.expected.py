from hyper import html


@html
def OnlyParams(*, name: str, count: int = 0, items: list):
    pass
