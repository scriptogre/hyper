"""Hyper - Python framework for hypermedia-driven applications.

Public API exports:
- @html decorator (from hyper.decorators)
- HTML helpers (from hyper.helpers)
- Content collections (from hyper.content, requires 'content' extra)
"""

# Decorators
from hyper.decorators import html

# HTML rendering helpers
from hyper.helpers import (
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
    # Decorator
    'html',
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
