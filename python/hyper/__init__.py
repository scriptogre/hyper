"""Hyper - Python framework for hypermedia-driven applications.

Public API exports:
- Component decorator (from hyper.component)
- HTML helpers (from hyper.html)
- Content collections (from hyper.content, requires 'content' extra)
"""

# Decorators
from hyper.decorators import component

# HTML rendering helpers
from hyper.html import (
    Safe,
    safe,
    escape_html,
    render_attr,
    render_class,
    render_style,
    render_data,
    render_aria,
    spread_attrs,
    replace_markers,
)

# Primary alias for transpiler - clear and readable
escape = escape_html

__all__ = [
    # Component decorator
    'component',
    # Core escaping
    'Safe',
    'safe',
    'escape',
    'escape_html',
    # Attribute rendering
    'render_attr',
    'render_class',
    'render_style',
    'render_data',
    'render_aria',
    'spread_attrs',
    'replace_markers',
]

# Content collections (optional, requires 'content' extra)
try:
    from hyper.content import (
        Collection,
        MarkdownCollection,
        MarkdownSingleton,
        Singleton,
        computed,
        load,
    )

    __all__.extend([
        'Collection',
        'MarkdownCollection',
        'MarkdownSingleton',
        'Singleton',
        'computed',
        'load',
    ])
except ImportError:
    # Content extra not installed
    pass
