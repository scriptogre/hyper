def CardWithDefaults(
        # Card with default parameter values
        title: str = "Untitled",
        content: str = "",
        variant: str = "default",
        show_border: bool = True,
):
    _parts = []
    _parts.append(f"""<div class="card card-{variant} {'bordered' if show_border else ''}">
    <h2>{title}</h2>""")
    if content:
        _parts.append(f"""        <p>{content}</p>""")
    else:
        _parts.append(f"""        <p class="empty">No content provided</p>""")
    _parts.append(f"""</div>""")
    return "".join(_parts)
