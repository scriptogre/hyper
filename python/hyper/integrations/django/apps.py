"""Django AppConfig that discovers Hyper components on app readiness."""

from __future__ import annotations

import importlib
import warnings
from pathlib import Path

from django.apps import AppConfig, apps
from django.conf import settings

from hyper.integrations._discovery import discover_in_package, discover_in_path
from hyper.integrations.django import _registry


class HyperConfig(AppConfig):
    """Walks INSTALLED_APPS at startup for ``<app>/components/`` directories.

    Also honours an optional ``HYPER_COMPONENT_PACKAGES`` Django setting (an
    iterable of dotted module paths) for components that don't live under an
    installed app's ``components/`` directory.
    """

    name = "hyper.integrations.django"
    label = "hyper"
    verbose_name = "Hyper"

    def ready(self) -> None:
        discovered: dict[str, object] = {}

        # 1. Walk each installed app for a `components/` subdirectory.
        for app_config in apps.get_app_configs():
            if app_config.name == self.name:
                continue
            components_dir = Path(app_config.path) / "components"
            if not components_dir.is_dir():
                continue
            for name, component in discover_in_path(components_dir):
                discovered[name] = component

        # 2. Optional override: HYPER_COMPONENT_PACKAGES = ["other.pkg", ...]
        extra = getattr(settings, "HYPER_COMPONENT_PACKAGES", ())
        for dotted in extra:
            try:
                package = importlib.import_module(dotted)
            except Exception as e:
                warnings.warn(
                    f"Hyper: HYPER_COMPONENT_PACKAGES entry {dotted!r} "
                    f"could not be imported: {e!r}",
                    stacklevel=2,
                )
                continue
            for name, component in discover_in_package(package):
                discovered[name] = component

        _registry.set_components(discovered)
