"""Django template tag for invoking Hyper components from DTE templates.

Usage:

    {% load hyper %}    {# omit if registered as a builtin in TEMPLATES OPTIONS #}

    {% hyper Sidebar user=user %}                 {# Sidebar resolved via context #}
    {% hyper "myapp.components.Sidebar" user=user %}   {# dotted-path fallback #}

The tag accepts either:
  - a callable already in the template context (the recommended path; populate
    via ``hyper.integrations.django.context_processors.components``), or
  - a string dotted path to import on demand.

The output is marked safe — Hyper components produce escaped HTML themselves.
"""

from __future__ import annotations

import importlib

from django import template
from django.utils.safestring import mark_safe

register = template.Library()


def _resolve(target):
    """Accept a callable directly, or a 'pkg.mod.Name' string to import."""
    if callable(target):
        return target
    if isinstance(target, str):
        if "." not in target:
            raise template.TemplateSyntaxError(
                f"{{% hyper %}}: {target!r} is a bare string, not a dotted "
                "import path. Either pass a component from context (set up "
                "the components context processor) or write the full path "
                "like 'myapp.components.Sidebar'."
            )
        module_path, _, attr = target.rpartition(".")
        try:
            module = importlib.import_module(module_path)
        except ImportError as e:
            raise template.TemplateSyntaxError(
                f"{{% hyper %}}: could not import {module_path!r}: {e}"
            ) from e
        try:
            return getattr(module, attr)
        except AttributeError as e:
            raise template.TemplateSyntaxError(
                f"{{% hyper %}}: {module_path!r} has no attribute {attr!r}"
            ) from e
    raise template.TemplateSyntaxError(
        f"{{% hyper %}}: expected a component or dotted import path, "
        f"got {type(target).__name__}"
    )


@register.simple_tag(name="hyper")
def hyper_tag(target, **kwargs):
    """Render a Hyper component, returning HTML-safe output."""
    component = _resolve(target)
    if not callable(component):
        raise template.TemplateSyntaxError(
            f"{{% hyper %}}: {component!r} is not callable"
        )
    return mark_safe(str(component(**kwargs)))
