"""Minimal Django AppConfig so the fixtures package can act as an installed app.

The Hyper Django integration walks every installed app for a ``components/``
subdirectory. This config exposes the fixtures package as ``fixtures`` with
``fixtures/components/`` carrying the test components.
"""

from django.apps import AppConfig


class FixturesConfig(AppConfig):
    name = "fixtures"
    label = "fixtures"
