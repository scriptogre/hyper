"""Hyper HTML template runtime.

Public API exports:
- @component decorator and Component type (from hyperhtml.decorators)
- HTML helpers (from hyperhtml.helpers)
- Content collections (from hyperhtml.content, requires 'content' extra)
"""

# Components
from hyperhtml.decorators import Component, component

# HTML rendering helpers
from hyperhtml.helpers import (
    Safe,
    safe,
    escape_html,
    render_attr,
    render_class,
    render_style,
    render_data,
    render_aria,
    spread_attrs,
)

# Primary alias for transpiler - clear and readable
escape = escape_html

__all__ = [
    # Components
    "Component",
    "component",
    # Core escaping
    "Safe",
    "safe",
    "escape",
    "escape_html",
    # Attribute rendering
    "render_attr",
    "render_class",
    "render_style",
    "render_data",
    "render_aria",
    "spread_attrs",
]

# Content collections (optional, requires 'content' extra)
try:
    from hyperhtml.content import (
        Collection,
        MarkdownCollection,
        MarkdownSingleton,
        Singleton,
        computed,
        load,
    )

    __all__.extend(
        [
            "Collection",
            "MarkdownCollection",
            "MarkdownSingleton",
            "Singleton",
            "computed",
            "load",
        ]
    )
except ImportError:
    # Content extra not installed
    pass
