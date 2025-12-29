# GENERATOR TEST: Parameters but no body
# Should generate: def __hyper_template__(a: str, b: int): pass

def GeneratorEdge_ParamsOnly(a: str, b: int = 0, c: bool = True):
    _parts = []
    # Only parameters above, no HTML or expressions
    return "".join(_parts)
