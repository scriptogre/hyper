"""Runtime utilities for generated template code.

These functions are called by compiled templates at render time.
Minimal dependencies - just value processing, no tree building.
"""

from typing import Any

from markupsafe import Markup, escape


def escape_html(value: Any) -> str:
    """Escape value for safe HTML output.

    - Markup objects: pass through unchanged (already safe)
    - None: empty string
    - Other: str() then escape
    """
    if isinstance(value, Markup):
        return value
    if value is None:
        return ""
    return escape(str(value))


def format_classes(*values: str | list | dict | None) -> str:
    """Convert class values to space-separated string.

    Accepts:
        - str: "btn primary"
        - list: ["btn", "primary"]
        - dict: {"active": True, "disabled": False} -> "active"
        - Nested combinations
        - None/False: ignored

    Example:
        format_classes("btn", ["large"], {"disabled": False})
        # Returns: "btn large"
    """
    classes: list[str] = []
    queue = list(values)

    while queue:
        value = queue.pop(0)

        if not value:  # None, False, empty string/list/dict
            continue

        if isinstance(value, str):
            classes.append(value)
        elif isinstance(value, dict):
            for key, enabled in value.items():
                if enabled:
                    classes.append(str(key))
        elif isinstance(value, (list, tuple)):
            # Add to front of queue to process in order
            queue[0:0] = value
        elif isinstance(value, bool):
            pass  # Ignore standalone booleans

    return " ".join(c.strip() for c in classes if c.strip())


def format_styles(styles: dict[str, str | None]) -> str:
    """Convert style dict to CSS string.

    Example:
        format_styles({"color": "red", "margin": None})
        # Returns: "color: red"
    """
    if isinstance(styles, str):
        return styles
    parts = []
    for key, value in styles.items():
        if value is not None:
            parts.append(f"{key}: {value}")
    return "; ".join(parts)


def format_attrs(attrs: dict[str, Any]) -> str:
    """Convert attribute dict to HTML attribute string.

    - True: attribute without value (e.g., disabled)
    - False/None: omit attribute
    - Other: name="escaped_value"

    Example:
        format_attrs({"disabled": True, "id": "main", "hidden": False})
        # Returns: ' disabled id="main"'
    """
    parts = []
    for key, value in attrs.items():
        if value is True:
            parts.append(f" {key}")
        elif value is False or value is None:
            pass  # Omit attribute
        else:
            escaped = escape_html(value)
            parts.append(f' {key}="{escaped}"')
    return "".join(parts)


def join_children(children: tuple) -> str:
    """Join children tuple to string."""
    return "".join(str(c) for c in children)


def render_data_attrs(data: dict[str, Any]) -> str:
    """Render data-* attributes from a dict.

    Example:
        render_data_attrs({"id": 123, "active": True})
        # Returns: ' data-id="123" data-active'
    """
    parts = []
    for key, value in data.items():
        attr_name = f"data-{key}"
        if value is True:
            parts.append(f" {attr_name}")
        elif value is False or value is None:
            pass
        else:
            escaped = escape_html(value)
            parts.append(f' {attr_name}="{escaped}"')
    return "".join(parts)


def render_aria_attrs(aria: dict[str, Any]) -> str:
    """Render aria-* attributes from a dict.

    Boolean values are converted to "true"/"false" strings per ARIA spec.

    Example:
        render_aria_attrs({"hidden": True, "label": "Close"})
        # Returns: ' aria-hidden="true" aria-label="Close"'
    """
    parts = []
    for key, value in aria.items():
        attr_name = f"aria-{key}"
        if value is True:
            parts.append(f' {attr_name}="true"')
        elif value is False:
            parts.append(f' {attr_name}="false"')
        elif value is None:
            pass
        else:
            escaped = escape_html(value)
            parts.append(f' {attr_name}="{escaped}"')
    return "".join(parts)
