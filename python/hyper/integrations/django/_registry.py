"""Internal registry for Hyper components discovered by the Django integration."""

from __future__ import annotations

_components: dict[str, object] = {}


def set_components(items: dict[str, object]) -> None:
    """Replace the registry contents."""
    _components.clear()
    _components.update(items)


def all_components() -> dict[str, object]:
    """Return a shallow copy of the registry. Safe to expose to template contexts."""
    return dict(_components)


def get(name: str):
    """Look up a component by name, or None when not registered."""
    return _components.get(name)
