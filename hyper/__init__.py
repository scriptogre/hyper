"""Hyper - A Python framework for hypermedia-driven applications."""


class _MissingExtra:
    """Placeholder for features that require an optional extra."""

    def __init__(self, name: str, extra: str):
        self._name = name
        self._extra = extra

    def __call__(self, *args, **kwargs):
        raise ImportError(
            f"{self._name} requires hyper[{self._extra}].\n\n"
            f"  Install it:\n"
            f"    uv add hyper[{self._extra}]"
        )


_TEMPLATE_EXPORTS = [
    "enable_templates",
    "slot",
    # Errors
    "TemplateError",
    "PropValidationError",
    "TemplateNotFoundError",
    "TemplateCompileError",
    "SlotError",
]

_CONTENT_EXPORTS = [
    "Collection",
    "MarkdownCollection",
    "MarkdownSingleton",
    "Singleton",
    "computed",
    "load",
]

# Templates — explicit imports for IDE support (except enable_templates which is lazy)
_templates_available = False
try:
    from hyper.templates import (  # noqa: F401
        TemplateError,
        PropValidationError,
        TemplateNotFoundError,
        TemplateCompileError,
        SlotError,
        slot,
    )

    _templates_available = True
except ImportError:
    for _name in _TEMPLATE_EXPORTS:
        if _name != "enable_templates":
            globals()[_name] = _MissingExtra(_name, "templates")

# Content — explicit imports for IDE support
try:
    from hyper.content import (  # noqa: F401
        Collection,
        MarkdownCollection,
        MarkdownSingleton,
        Singleton,
        computed,
        load,
    )
except ImportError:
    for _name in _CONTENT_EXPORTS:
        globals()[_name] = _MissingExtra(_name, "content")


def __getattr__(name: str):
    """Lazy access to enable_templates to trigger _do_enable() on each import."""
    if name == "enable_templates":
        if _templates_available:
            from hyper.templates import _enable_templates_instance

            _enable_templates_instance._do_enable()
            return _enable_templates_instance
        else:
            return _MissingExtra("enable_templates", "templates")
    raise AttributeError(f"module 'hyper' has no attribute '{name}'")


__all__ = _TEMPLATE_EXPORTS + _CONTENT_EXPORTS
