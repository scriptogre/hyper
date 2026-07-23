"""Tests for Hyper components rendering through Django's Jinja2 template backend.

Verifies the canonical Django+Jinja2 wiring: HyperExtension is declared in
TEMPLATES['OPTIONS']['extensions'] and auto-discovers components from the
backend's template directories.
"""

from __future__ import annotations

from pathlib import Path

import django
import pytest
from django.conf import settings


@pytest.fixture
def django_jinja_setup(components_dir: Path):
    """Configure Django with the Jinja2 backend pointing at the fixtures directory."""
    if settings.configured:
        # Reset so we can reconfigure for this test.
        from django.test.utils import override_settings

        override = override_settings(
            TEMPLATES=[
                {
                    "BACKEND": "django.template.backends.jinja2.Jinja2",
                    "DIRS": [str(components_dir)],
                    "OPTIONS": {
                        "extensions": ["hyperhtml.integrations.jinja2.HyperExtension"],
                    },
                },
            ]
        )
        override.enable()
        yield
        override.disable()
        return

    settings.configure(
        DEBUG=True,
        TEMPLATES=[
            {
                "BACKEND": "django.template.backends.jinja2.Jinja2",
                "DIRS": [str(components_dir)],
                "OPTIONS": {
                    "extensions": ["hyperhtml.integrations.jinja2.HyperExtension"],
                },
            },
        ],
        INSTALLED_APPS=[],
    )
    django.setup()
    yield


def test_jinja_backend_renders_hyper_component(django_jinja_setup):
    from django.template import engines

    jinja_backend = engines["jinja2"]
    template = jinja_backend.from_string("{{ Greeting(name='Ada') }}")

    out = template.render()
    assert out == "<p>Hello, Ada!</p>"


def test_jinja_backend_escapes_other_context(django_jinja_setup):
    from django.template import engines

    jinja_backend = engines["jinja2"]
    template = jinja_backend.from_string("{{ raw }} | {{ Greeting(name='Ada') }}")

    out = template.render({"raw": "<script>"})
    assert "&lt;script&gt;" in out
    assert "<p>Hello, Ada!</p>" in out


def test_jinja_backend_renders_card_with_two_args(django_jinja_setup):
    from django.template import engines

    jinja_backend = engines["jinja2"]
    template = jinja_backend.from_string('{{ Card(title="Hi", body="There") }}')

    out = template.render()
    assert "<h2>Hi</h2>" in out
    assert "<p>There</p>" in out
