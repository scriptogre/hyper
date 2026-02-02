from hyper import component


@component
def OnlyParams(*, name: str, count: int = 0, items: list):
