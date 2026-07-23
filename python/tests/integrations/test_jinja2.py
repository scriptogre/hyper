"""Tests for hyperhtml.integrations.jinja2.HyperExtension.

Verifies that adding the extension to a Jinja2 env auto-discovers Hyper
components living next to Jinja templates, exposes them as globals,
and the rendered output is treated as safe HTML (no double-escaping).
"""

from __future__ import annotations

import warnings
from pathlib import Path

import pytest
from jinja2 import (
    ChoiceLoader,
    DictLoader,
    Environment,
    FileSystemLoader,
    PackageLoader,
    select_autoescape,
)

from hyperhtml import component
from hyperhtml.integrations.jinja2 import HyperExtension


def _env(loader, autoescape=True):
    env = Environment(
        loader=loader,
        autoescape=select_autoescape(default=autoescape),
    )
    env.add_extension(HyperExtension)
    return env


def test_autodiscovers_components_from_filesystem_loader(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))

    assert "Greeting" in env.globals
    assert "Card" in env.globals
    assert callable(env.globals["Greeting"])


def test_component_renders_into_jinja_template_unescaped(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))
    template = env.from_string("{{ Greeting(name='Ada') }}")

    out = template.render()

    # Hyper output reached Jinja as safe HTML via __html__; angle brackets intact.
    assert out == "<p>Hello, Ada!</p>"


def test_jinja_still_escapes_other_variables(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))
    template = env.from_string("{{ raw }} | {{ Greeting(name='Ada') }}")

    out = template.render(raw="<script>")

    assert out.startswith("&lt;script&gt;")
    assert "<p>Hello, Ada!</p>" in out


def test_hyper_escapes_user_input_inside_component(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))
    template = env.from_string("{{ Greeting(name=raw) }}")

    out = template.render(raw="<script>")

    # The component itself escapes its inputs (via escape()), so user-supplied
    # html in arguments is safe even though the component output is safe-marked.
    assert "<script>" not in out
    assert "&lt;script&gt;" in out


def test_choice_loader_walks_all_search_paths(tmp_path, components_dir: Path):
    other_dir = tmp_path / "other"
    other_dir.mkdir()
    # Empty extra path; finder should still discover from components_dir.

    env = _env(
        ChoiceLoader(
            [
                FileSystemLoader(str(other_dir)),
                FileSystemLoader(str(components_dir)),
            ]
        )
    )

    assert "Greeting" in env.globals
    assert "Card" in env.globals


def test_warns_on_non_introspectable_loader_and_skips_discovery():
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        env = _env(DictLoader({"index.html": "ok"}))

    msgs = [str(w.message) for w in caught]
    assert any("no introspectable filesystem paths" in m for m in msgs), msgs
    assert "Greeting" not in env.globals


def test_register_components_with_callable():
    env = _env(DictLoader({"index.html": "{{ Custom(x='hi') }}"}))

    @component
    def Custom(*, x: str):
        yield f"<b>{x}</b>"

    env.register_components(Custom)

    out = env.get_template("index.html").render()
    assert out == "<b>hi</b>"


def test_register_components_with_iterable():
    env = _env(DictLoader({}))

    @component
    def A(*, v: str):
        yield f"<a>{v}</a>"

    @component
    def B(*, v: str):
        yield f"<b>{v}</b>"

    env.register_components([A, B])

    assert env.globals["A"](v="x") == "<a>x</a>"
    assert env.globals["B"](v="y") == "<b>y</b>"


def test_register_components_with_package(components_dir: Path, monkeypatch):
    # Use the discovery-from-package code path via the test fixtures package.
    import sys

    fixtures_root = components_dir.parent.parent
    monkeypatch.syspath_prepend(str(fixtures_root))
    # Force a fresh import so __hyper__ markers are present.
    for name in list(sys.modules):
        if name.startswith("fixtures.components") or name == "fixtures":
            sys.modules.pop(name, None)

    import importlib

    pkg = importlib.import_module("fixtures.components")

    env = _env(DictLoader({}))
    env.register_components(pkg)

    assert "Greeting" in env.globals
    assert "Card" in env.globals


def test_register_components_rejects_garbage():
    env = _env(DictLoader({}))
    with pytest.raises(TypeError):
        env.register_components(42)


# --- {% hyper %} / {% slot %} ------------------------------------------------


def test_hyper_tag_fills_default_and_named_slots(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))
    template = env.from_string(
        "{% hyper Panel(title='Pricing') %}"
        "<p>Body</p>"
        '{% slot actions %}<a href="/buy">Buy</a>{% endslot %}'
        "{% endhyper %}"
    )
    out = template.render()

    assert "<h2>Pricing</h2>" in out
    assert "<p>Body</p>" in out
    assert '<a href="/buy">Buy</a>' in out
    assert "No actions" not in out  # named slot replaced the fallback


def test_slots_work_in_an_async_environment():
    # The slot rewrite is plain Jinja, so an async env + async component just
    # works: AssignBlock captures and the call all compile to async code.
    import asyncio

    @component
    async def Panel(*, title, content=None, actions=None):
        yield f"<h2>{title}</h2>"
        for s in actions or ["No actions"]:
            yield s

    env = Environment(loader=DictLoader({}), enable_async=True)
    env.add_extension(HyperExtension)
    env.globals["Panel"] = Panel

    template = env.from_string(
        "{% hyper Panel(title='Pricing') %}"
        "{% slot actions %}<a>Buy</a>{% endslot %}"
        "{% endhyper %}"
    )
    out = asyncio.run(template.render_async())

    assert out == "<h2>Pricing</h2><a>Buy</a>"


def test_slot_outside_hyper_is_a_syntax_error(components_dir: Path):
    # {% slot %} only means something inside a {% hyper %} block. Used loose, it
    # fails at parse time rather than miscompiling.
    from jinja2 import TemplateSyntaxError

    env = _env(FileSystemLoader(str(components_dir)))
    with pytest.raises(TemplateSyntaxError, match="inside a .* hyper"):
        env.from_string("{% slot actions %}<a>Buy</a>{% endslot %}")


def test_omitted_named_slot_renders_fallback(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))
    template = env.from_string(
        "{% hyper Panel(title='Pricing') %}<p>Body</p>{% endhyper %}"
    )
    out = template.render()

    assert "<p>Body</p>" in out
    assert "No actions" in out  # actions slot not supplied -> fallback


def test_blank_body_leaves_default_slot_empty(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))
    template = env.from_string("{% hyper Panel(title='Pricing') %}   {% endhyper %}")
    out = template.render()

    assert "<h2>Pricing</h2>" in out
    assert "No actions" in out


def test_hyper_tag_spreads_a_dict(components_dir: Path):
    env = _env(FileSystemLoader(str(components_dir)))
    template = env.from_string("{% hyper Panel(**props) %}<p>Body</p>{% endhyper %}")
    out = template.render(props={"title": "Pricing"})

    assert "<h2>Pricing</h2>" in out


def test_hyper_tag_rejects_non_call_expression(components_dir: Path):
    from jinja2.exceptions import TemplateSyntaxError

    env = _env(FileSystemLoader(str(components_dir)))
    with pytest.raises(TemplateSyntaxError):
        env.from_string("{% hyper Panel %}{% endhyper %}")
