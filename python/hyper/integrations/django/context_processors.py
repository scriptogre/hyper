"""Django context processors for Hyper components."""

from __future__ import annotations

from hyper.integrations.django import _registry


def components(request):
    """Inject every discovered Hyper component into the template context.

    Add to your DTE backend's ``OPTIONS["context_processors"]`` to make
    ``{% hyper Sidebar user=user %}`` resolve without per-view wiring.
    """
    return _registry.all_components()
