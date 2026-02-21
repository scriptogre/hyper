"""HTML escaping and rendering helpers for Hyper templates.

These functions are used by compiled templates to safely render HTML.
"""

import re
import ast

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
    'replace_markers',
]

# Compiled regex for attribute markers in templates
# Matches both:
# - attrname=‹TYPE:value› (for BOOL, CLASS, STYLE, DATA, ARIA)
# - ‹TYPE:value› (for SPREAD)
_ATTR_MARKER_PATTERN = re.compile(r'(?:(\w+)=)?‹(\w+):(.+?)›')

# Pattern for ESCAPE markers: ‹ESCAPE:{expr}›
# These are standalone markers (not attributes) that need HTML escaping
# When the f-string runs, {expr} is evaluated and we get ‹ESCAPE:value›
_ESCAPE_MARKER_PATTERN = re.compile(r'‹ESCAPE:(.+?)›')


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


def replace_markers(html: str) -> str:
    """Replace attribute markers in compiled template HTML.

    Processes markers like attrname=‹TYPE:value› and replaces them with
    properly rendered attributes. This allows compiled templates to keep
    the same structure as source .hyper files for better IDE mapping.

    Supported marker types:
    - BOOL: Boolean attributes (True → attr name only, False → removed entirely)
    - CLASS: Class lists/dicts processed via render_class()
    - STYLE: Style dicts processed via render_style()
    - DATA: Data attribute dicts expanded to data-* attributes
    - ARIA: ARIA attribute dicts expanded to aria-* attributes
    - SPREAD: Dict spread as multiple attributes
    - ESCAPE: Expressions that need HTML escaping

    Args:
        html: HTML string containing attribute markers.

    Returns:
        Processed HTML with markers replaced.

    Example:
        >>> replace_markers('<button disabled=‹BOOL:True›>')
        '<button disabled>'
        >>> replace_markers('<button disabled=‹BOOL:False›>')
        '<button>'
        >>> replace_markers('<div class=‹CLASS:["btn", "active"]›>')
        '<div class="btn active">'
        >>> replace_markers('<div data=‹DATA:{"user-id": 123}›>')
        '<div data-user-id="123">'
        >>> replace_markers('‹ESCAPE:{user_input}›')
        '<escaped output>'
    """
    # First, process ESCAPE markers (standalone, not attributes)
    def replace_escape(match):
        # The f-string has already evaluated {expr}, so we get the actual value here
        # E.g., source: {item} → compiled: ‹ESCAPE:{item}› → f-string evaluates to: ‹ESCAPE:actual_value›
        value = match.group(1)
        return escape_html(str(value))

    html = _ESCAPE_MARKER_PATTERN.sub(replace_escape, html)

    # Then process attribute markers
    def replace(match):
        attr_name, marker_type, value_str = match.groups()

        # Parse the value using ast.literal_eval for safety
        try:
            value = ast.literal_eval(value_str)
        except (ValueError, SyntaxError):
            # If parsing fails, return original marker
            return match.group(0)

        # Process based on marker type
        match marker_type:
            case 'BOOL':
                # Boolean: True → just attr name, False → remove entirely
                return attr_name if value else ''

            case 'CLASS':
                # Class: render as class="..."
                return f'{attr_name}="{render_class(value)}"'

            case 'STYLE':
                # Style: render as style="..."
                return f'{attr_name}="{render_style(value)}"'

            case 'DATA':
                # Data: expand to data-* attributes (removes "data=" prefix)
                return render_data(value).lstrip()

            case 'ARIA':
                # ARIA: expand to aria-* attributes (removes "aria=" prefix)
                return render_aria(value).lstrip()

            case 'SPREAD':
                # Spread: expand dict to attributes (no attribute name in source)
                return spread_attrs(value).lstrip()

            case _:
                # Unknown marker type, keep as-is
                return match.group(0)

    return _ATTR_MARKER_PATTERN.sub(replace, html)
