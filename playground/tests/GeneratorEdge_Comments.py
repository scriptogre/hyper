# GENERATOR TEST: Comments in various positions

# Header comment 1
# Header comment 2

"""
This is a docstring
"""

# test

def GeneratorEdge_Comments(
        # Inline comment on parameter
        name: str,
        # Another comment
        count: int,
):
    _parts = []
    # Comment after parameter, before body
    _parts.append(f"""<div>""")
    # Comment inside body
    _parts.append(f"""    <span>{name}</span>""")
    # Another body comment
    _parts.append(f"""</div>
""")
    # Trailing comment
    return "".join(_parts)
