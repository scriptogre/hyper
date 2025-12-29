def FormField(
        # Reusable form field component
        name: str,
        label: str,
        type: str = "text",
        value: str = "",
        error: str | None = None,
        required: bool = False,
        help_text: str = "",
):
    _parts = []
    _parts.append(f"""<div class="form-field {'has-error' if error else ''}">
    <label for="{name}">
        {label}""")
    if required:
        _parts.append(f"""            <span class="required">*</span>""")
    _parts.append(f"""    </label>

    <input
        type="{type}"
        id="{name}"
        name="{name}"
        value="{value}"
        required="{required}"
        aria-describedby="{name}-help {name}-error"/>
""")
    if error:
        _parts.append(f"""        <span id="{name}-error" class="field-error" role="alert">
            {error}
        </span>""")
    elif help_text:
        _parts.append(f"""        <span id="{name}-help" class="field-help">
            {help_text}
        </span>""")
    _parts.append(f"""</div>""")
    return "".join(_parts)
