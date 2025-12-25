from hyper.templates.runtime import (
    escape_html,
    format_classes,
    format_styles,
    format_attrs,
    render_data_attrs,
    render_aria_attrs,
)
from markupsafe import Markup


def render(variant: str, type: str, __children__: tuple = (), __attrs__: dict = {}) -> str:
    return '
' + f'<button{f' class="btn btn-{escape_html(variant)}"' + f' _="on click set my.checked to {escape_html(variant != "disabled")}"' + format_attrs(__slot__)}>{f'
    {escape_html(__slot__)}
'}</button>' + '
'


# Public API
__call__ = render