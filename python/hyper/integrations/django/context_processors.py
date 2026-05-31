"""
Django context processor for Hyper components.
"""

from __future__ import annotations

from django.apps import apps


def components(request):
    """
    Put every discovered component into the template context, so any
    template can call one by name:

        {% hyper Sidebar user=user / %}
    """
    # {"Sidebar": <Sidebar component>, "Card": <Card component>, ...}
    return dict(apps.get_app_config("hyper").components)
