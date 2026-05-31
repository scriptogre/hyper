"""Django AppConfig that discovers Hyper components on app readiness."""

from __future__ import annotations

from pathlib import Path

from django.apps import AppConfig
from django.template import engines
from django.template.backends.django import DjangoTemplates
from django.template.utils import get_app_template_dirs

from hyper.integrations._discovery import discover_in_path


class HyperConfig(AppConfig):
    """Discover Hyper components at startup, wherever Django looks for templates.

    Reuses each ``DjangoTemplates`` backend's own resolved template directories,
    so Hyper looks in exactly the places that backend does: the ``DIRS`` you
    list, plus every app's ``templates/`` dir when ``APP_DIRS`` is on.

    Drop a ``.hyper`` file (compiled to its ``.py`` sibling) anywhere you would
    put a template, and it's discovered. No configuration. Read results via
    ``apps.get_app_config("hyper").components``.
    """

    name = "hyper.integrations.django"
    label = "hyper"
    verbose_name = "Hyper"

    components: dict[str, object] = {}

    def ready(self) -> None:
        components: dict[str, object] = {}
        seen: set[Path] = set()

        for backend in engines.all():
            if not isinstance(backend, DjangoTemplates):
                continue
            engine = backend.engine
            directories = list(engine.dirs)
            if engine.app_dirs:
                directories += get_app_template_dirs("templates")
            for directory in directories:
                resolved = Path(directory).resolve()
                if resolved in seen:
                    continue
                seen.add(resolved)
                components.update(discover_in_path(resolved))

        self.components = components
