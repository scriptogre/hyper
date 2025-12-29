def ComplexTypes(
        # Tests complex type annotations in parameters
        items: list[dict[str, Any]],
        config: dict[str, list[int]],
        callback: Callable[[str, int], bool],
        optional_user: Optional[dict] = None,
        union_value: int | str | None = None,
):
    _parts = []
    _parts.append(f"""<div class="complex-types">
    <span>Items count: {len(items)}</span>
    <span>Config keys: {list(config.keys())}</span>""")
    if optional_user:
        _parts.append(f"""        <span>User: {optional_user.get('name')}</span>""")
    _parts.append(f"""    <span>Union value: {union_value}</span>
</div>""")
    return "".join(_parts)
