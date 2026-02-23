"""HTML escaping and rendering helpers for Hyper templates.

These functions are used by compiled templates to safely render HTML.
"""

import re

__all__ = [
    'Safe',
    'safe',
    'escape_html',
    'render_attr',
    'render_class',
    'render_style',
    'render_data',
    'render_aria',
    'spread_attrs',
]


class Safe(str):
    """A string marked as safe HTML (will not be escaped).

    Use the safe() function to create instances.
    """

    def __html__(self):
        return self


def safe(value) -> Safe:
    """Mark a value as safe HTML that should not be escaped.

    Args:
        value: The value to mark as safe. Can be a string, an object
               with __html__ method, or None.

    Returns:
        A Safe string that won't be escaped when rendered.

    Example:
        >>> safe("<b>bold</b>")
        Safe('<b>bold</b>')
        >>> safe(None)
        Safe('')
    """
    if value is None:
        return Safe('')
    if hasattr(value, '__html__'):
        return Safe(value.__html__())
    return Safe(str(value))


def escape_html(value) -> str:
    """Escape a value for safe HTML output.

    Replaces HTML special characters with their entity equivalents:
    - & → &amp;
    - < → &lt;
    - > → &gt;
    - " → &quot;
    - ' → &#x27;

    If the value has an __html__ method (like Safe), returns that directly
    without escaping.

    Args:
        value: The value to escape. Can be any type.

    Returns:
        Escaped HTML string.

    Example:
        >>> escape_html("<script>alert('XSS')</script>")
        "&lt;script&gt;alert(&#x27;XSS&#x27;)&lt;/script&gt;"
        >>> escape_html(safe("<b>bold</b>"))
        "<b>bold</b>"
    """
    if value is None:
        return ''
    if hasattr(value, '__html__'):
        return value.__html__()

    s = str(value)
    return (s
        .replace('&', '&amp;')
        .replace('<', '&lt;')
        .replace('>', '&gt;')
        .replace('"', '&quot;')
        .replace("'", '&#x27;'))


def render_attr(name: str, value) -> str:
    """Render a single HTML attribute.

    Handles boolean attributes and dynamic values:
    - True: renders just the attribute name (e.g., "disabled")
    - False/None: renders nothing
    - Other values: renders name="escaped_value"

    Args:
        name: The attribute name.
        value: The attribute value.

    Returns:
        Rendered attribute string with leading space, or empty string.

    Example:
        >>> render_attr("disabled", True)
        ' disabled'
        >>> render_attr("disabled", False)
        ''
        >>> render_attr("id", "main")
        ' id="main"'
    """
    if value is True:
        return f' {name}'
    if value is False or value is None:
        return ''
    return f' {name}="{escape_html(value)}"'


def render_class(*values) -> str:
    """Render a class attribute value from various inputs.

    Accepts:
    - str: passed through as-is
    - list/tuple: items joined with spaces (nested structures supported)
    - dict: keys included if values are truthy
    - Multiple arguments: combined together

    Args:
        *values: One or more class values to render.

    Returns:
        Space-separated class names.

    Example:
        >>> render_class("btn", "primary")
        'btn primary'
        >>> render_class(["btn", "large"])
        'btn large'
        >>> render_class({"active": True, "disabled": False})
        'active'
        >>> render_class("btn", {"active": True}, ["lg"])
        'btn active lg'
    """
    classes = []
    queue = list(values)

    while queue:
        value = queue.pop(0)
        if not value:
            continue
        if isinstance(value, str):
            classes.append(value)
        elif isinstance(value, dict):
            classes.extend(k for k, v in value.items() if v)
        elif isinstance(value, (list, tuple)):
            queue[0:0] = list(value)

    return ' '.join(classes)


def render_style(value) -> str:
    """Render a style attribute value.

    Accepts:
    - str: passed through as-is
    - dict: rendered as "key:value;key:value"
    - None: returns empty string

    Args:
        value: The style value to render.

    Returns:
        CSS style string.

    Example:
        >>> render_style({"color": "red", "font-size": "14px"})
        'color:red;font-size:14px'
        >>> render_style("color: blue")
        'color: blue'
    """
    if value is None:
        return ''
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        return ';'.join(f'{k}:{v}' for k, v in value.items() if v is not None)
    return str(value) if value else ''


def render_data(attrs: dict) -> str:
    """Render data attributes from a dictionary.

    Each key-value pair is rendered as data-key="value".

    Args:
        attrs: Dictionary of data attribute names to values.

    Returns:
        Rendered data attributes with leading spaces.

    Example:
        >>> render_data({"user-id": 123, "role": "admin"})
        ' data-user-id="123" data-role="admin"'
        >>> render_data({})
        ''
    """
    if not attrs:
        return ''
    return ''.join(f' data-{k}="{escape_html(v)}"' for k, v in attrs.items() if v is not None)


def render_aria(attrs: dict) -> str:
    """Render ARIA attributes from a dictionary.

    Each key-value pair is rendered as aria-key="value".
    Boolean values are converted to "true" or "false" per ARIA spec.

    Args:
        attrs: Dictionary of ARIA attribute names to values.

    Returns:
        Rendered ARIA attributes with leading spaces.

    Example:
        >>> render_aria({"label": "Close dialog", "hidden": True})
        ' aria-label="Close dialog" aria-hidden="true"'
        >>> render_aria({"hidden": False})
        ' aria-hidden="false"'
    """
    if not attrs:
        return ''
    parts = []
    for k, v in attrs.items():
        if v is None:
            continue
        # Convert boolean values to "true"/"false" strings per ARIA spec
        if isinstance(v, bool):
            v = 'true' if v else 'false'
        parts.append(f' aria-{k}="{escape_html(v)}"')
    return ''.join(parts)


def spread_attrs(attrs: dict) -> str:
    """Spread a dictionary as HTML attributes.

    Each key-value pair is rendered as an attribute using render_attr().

    Args:
        attrs: Dictionary of attribute names to values.

    Returns:
        Rendered attributes with leading spaces.

    Example:
        >>> spread_attrs({"class": "btn", "id": "submit", "disabled": True})
        ' class="btn" id="submit" disabled'
        >>> spread_attrs({})
        ''
    """
    if not attrs:
        return ''
    return ''.join(render_attr(k, v) for k, v in attrs.items())
