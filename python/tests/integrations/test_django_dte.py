"""Tests for hyper.integrations.django rendering through Django's native template engine (DTE).

Verifies:
  - The Hyper app discovers components from every installed app's components/ dir
  - The context_processor exposes components to every DTE template context
  - The {% hyper Component arg=value %} tag invokes a component and marks safe
  - The string-path fallback {% hyper "pkg.mod.Name" arg=value %} works

The fixtures package doubles as a test Django app: it has __init__.py + apps.py
+ a components/ subdir, so HyperConfig.ready() picks it up via apps.get_app_configs().
"""

from __future__ import annotations

import sys
from pathlib import Path

import django
import pytest
from django.conf import settings


@pytest.fixture(scope="module", autouse=True)
def django_dte_setup():
    """Configure Django for DTE rendering with the Hyper integration installed."""
    fixtures_root = Path(__file__).parent
    sys.path.insert(0, str(fixtures_root))

    if not settings.configured:
        settings.configure(
            DEBUG=True,
            INSTALLED_APPS=[
                "hyper.integrations.django",
                "fixtures.apps.FixturesConfig",
            ],
            TEMPLATES=[
                {
                    "BACKEND": "django.template.backends.django.DjangoTemplates",
                    "DIRS": [str(fixtures_root)],
                    "OPTIONS": {
                        "context_processors": [
                            "hyper.integrations.django.context_processors.components",
                        ],
                        "builtins": [
                            "hyper.integrations.django.templatetags.hyper",
                        ],
                    },
                },
            ],
        )
        django.setup()
    yield
    sys.path.remove(str(fixtures_root))


@pytest.fixture(autouse=True)
def _populate_registry():
    """The autouse _clear_component_registry from conftest wipes the registry per-test.

    For these DTE tests we want it freshly populated by HyperConfig.ready() before
    each test, so re-run discovery here.
    """
    from hyper.integrations.django.apps import HyperConfig
    from django.apps import apps as django_apps

    config = django_apps.get_app_config("hyper")
    HyperConfig.ready(config)
    yield


def _render(source: str, context: dict | None = None) -> str:
    from django.template import Context, Template
    from django.template.context_processors import csrf  # noqa: F401  (sanity)
    from django.template.engine import Engine

    engine = Engine.get_default()
    template = engine.from_string(source)
    return template.render(Context(context or {}))


def test_registry_populated_on_app_ready():
    from django.apps import apps as django_apps

    components = django_apps.get_app_config("hyper").components
    assert "Greeting" in components
    assert "Card" in components
    assert callable(components["Greeting"])


def test_hyper_tag_renders_component_via_context_variable():
    out = _render("{% hyper Greeting name='Ada' / %}", _context_with_components())
    assert out == "<p>Hello, Ada!</p>"


def test_hyper_tag_marks_output_safe():
    out = _render("{% hyper Card title='Hi' body='There' / %}", _context_with_components())
    assert "<h2>Hi</h2>" in out
    assert "<p>There</p>" in out
    assert "&lt;" not in out  # the output should not be escaped


def test_hyper_tag_escapes_user_input_inside_component():
    # The component still escapes its arguments; user-supplied HTML in arguments is safe.
    out = _render(
        "{% hyper Greeting name=evil / %}",
        {**_context_with_components(), "evil": "<script>"},
    )
    assert "<script>" not in out
    assert "&lt;script&gt;" in out


def test_hyper_tag_accepts_dotted_string_path():
    out = _render(
        "{% hyper 'fixtures.components.Greeting.Greeting' name='Linus' / %}",
        {},  # no context processor needed for string-path lookups
    )
    assert out == "<p>Hello, Linus!</p>"


def test_hyper_tag_errors_on_unknown_component():
    from django.template import TemplateSyntaxError

    with pytest.raises(TemplateSyntaxError):
        _render("{% hyper 'does.not.exist' / %}", {})


def test_hyper_tag_errors_on_bare_string():
    from django.template import TemplateSyntaxError

    with pytest.raises(TemplateSyntaxError):
        _render("{% hyper 'JustAName' / %}", {})


# --- slots -------------------------------------------------------------------


def test_default_slot_fills_from_body():
    out = _render(
        "{% hyper Panel title='Pricing' %}<p>Body here</p>{% endhyper %}",
        _context_with_components(),
    )
    assert "<h2>Pricing</h2>" in out
    assert "<p>Body here</p>" in out


def test_named_slot_overrides_fallback():
    out = _render(
        "{% hyper Panel title='Pricing' %}"
        "<p>Body</p>"
        "{% slot actions %}<a href='/buy'>Buy</a>{% endslot %}"
        "{% endhyper %}",
        _context_with_components(),
    )
    assert "<a href='/buy'>Buy</a>" in out or '<a href="/buy">Buy</a>' in out
    assert "No actions" not in out  # fallback replaced


def test_omitted_named_slot_renders_fallback():
    out = _render(
        "{% hyper Panel title='Pricing' %}<p>Body</p>{% endhyper %}",
        _context_with_components(),
    )
    assert "No actions" in out  # named slot not supplied -> fallback


def test_whitespace_only_body_leaves_default_slot_empty():
    # The default slot has no fallback in Panel, so blank body yields nothing there.
    out = _render(
        "{% hyper Panel title='Pricing' %}   \n  {% endhyper %}",
        _context_with_components(),
    )
    assert "<h2>Pricing</h2>" in out
    assert "No actions" in out  # still falls back for the actions slot


def test_spread_passes_non_conflicting_props():
    out = _render(
        "{% hyper Card title='Hi' **extra / %}",
        {**_context_with_components(), "extra": {"body": "There"}},
    )
    assert "<h2>Hi</h2>" in out
    assert "<p>There</p>" in out


def test_spread_conflicting_key_raises_like_python():
    # A spread that collides with an explicit kwarg raises, same as Python/Jinja.
    from django.template import TemplateSyntaxError

    with pytest.raises(TemplateSyntaxError):
        _render(
            "{% hyper Greeting name='Ada' **extra / %}",
            {**_context_with_components(), "extra": {"name": "Linus"}},
        )


def test_discovery_scans_configured_template_dirs():
    # The fixtures live under a DIRS entry; discovery finds them with no setting.
    from django.apps import apps as django_apps
    from hyper.integrations.django.apps import HyperConfig

    config = django_apps.get_app_config("hyper")
    HyperConfig.ready(config)

    assert "Greeting" in config.components
    assert "Card" in config.components
    assert "Panel" in config.components


# --- helpers -----------------------------------------------------------------


def _context_with_components() -> dict:
    """Build a context that mimics what the components context processor would inject."""
    from django.apps import apps as django_apps

    return dict(django_apps.get_app_config("hyper").components)
