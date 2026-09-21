from __future__ import annotations

import inspect
import sys
from pathlib import Path

import pytest

pytest.importorskip("hyperhtml._native")

from hyperhtml import _autohook, _native


@pytest.fixture(autouse=True)
def hyper_imports():
    _autohook._uninstall_finder()
    _autohook._install_finder()
    yield
    _autohook._uninstall_finder()
    for name in list(sys.modules):
        if name == "app" or name.startswith("app."):
            sys.modules.pop(name, None)


def write(path: Path, source: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source)


def test_implicit_component_props_are_keyword_only(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "app" / "pages" / "Page.hyper",
        """title: str
---
<h1>{title}</h1>
""",
    )

    from app.pages import Page

    assert Page(title="Home").render() == "<h1>Home</h1>"
    with pytest.raises(TypeError):
        Page("Home")
    assert (
        inspect.signature(Page).parameters["title"].kind
        is inspect.Parameter.KEYWORD_ONLY
    )


def test_declared_component_props_and_spread_are_keyword_only(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "app" / "components" / "controls.hyper",
        """def Button(
    *,
    label: str,
    kind: str = "button",
    disabled: bool = False,
    **attrs,
) -> Component:
    <button>{label}</button>
""",
    )

    from app.components.controls import Button
    from hyper import Component

    signature = inspect.signature(Button)
    assert isinstance(Button, Component)
    assert signature.parameters["label"].kind is inspect.Parameter.KEYWORD_ONLY
    assert signature.parameters["kind"].kind is inspect.Parameter.KEYWORD_ONLY
    assert signature.parameters["disabled"].kind is inspect.Parameter.KEYWORD_ONLY
    assert signature.parameters["attrs"].kind is inspect.Parameter.VAR_KEYWORD
    assert (
        Button(label="Save", disabled=True, id="save").render()
        == "<button>Save</button>"
    )
    with pytest.raises(TypeError):
        Button("Save")


def test_implicit_and_declared_slot_signatures_match(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    body = """<aside>{...sidebar}</aside>
<main>{...}</main>
"""
    write(
        tmp_path / "app" / "pages" / "Page.hyper",
        f"""title: str
---
{body}""",
    )
    write(
        tmp_path / "app" / "components" / "layout.hyper",
        """def Layout(*, title: str) -> Component:
    <aside>{...sidebar}</aside>
    <main>{...}</main>
""",
    )

    from app.components.layout import Layout
    from app.pages import Page

    page_signature = inspect.signature(Page)
    layout_signature = inspect.signature(Layout)
    assert list(page_signature.parameters) == ["title", "content", "sidebar"]
    assert list(layout_signature.parameters) == ["title", "content", "sidebar"]
    assert all(
        parameter.kind is inspect.Parameter.KEYWORD_ONLY
        for parameter in page_signature.parameters.values()
    )
    assert all(
        parameter.kind is inspect.Parameter.KEYWORD_ONLY
        for parameter in layout_signature.parameters.values()
    )
    assert (
        Page(title="Home", content=[], sidebar=[]).render()
        == "<aside></aside><main></main>"
    )
    assert (
        Layout(title="Home", content=[], sidebar=[]).render()
        == "<aside></aside><main></main>"
    )


@pytest.mark.parametrize(
    "source",
    [
        """content: str
---
<p>{content}</p>
""",
        """def Panel(*, content: str) -> Component:
    <p>{content}</p>
""",
    ],
)
def test_content_is_reserved_for_the_default_slot(source):
    with pytest.raises(Exception) as caught:
        _native.transpile(source, "Panel.hyper")

    message = str(caught.value)
    assert "content" in message
    assert "default slot" in message.lower()
    assert "reserved" in message.lower()


@pytest.mark.parametrize(
    "source",
    [
        """header: str
---
<header>{...header}</header>
""",
        """def Panel(*, header: str) -> Component:
    <header>{...header}</header>
""",
    ],
)
def test_prop_cannot_share_a_named_slot_name(source):
    with pytest.raises(Exception) as caught:
        _native.transpile(source, "Panel.hyper")

    message = str(caught.value)
    assert "header" in message
    assert "prop" in message.lower()
    assert "named slot" in message.lower()


def test_named_slot_cannot_use_reserved_content_name():
    with pytest.raises(Exception) as caught:
        _native.transpile(
            """---
<main>{...content}</main>
""",
            "Panel.hyper",
        )

    message = str(caught.value)
    assert "content" in message
    assert "default slot" in message.lower()
    assert "named slot" in message.lower()


def test_declared_component_props_require_explicit_star():
    with pytest.raises(Exception) as caught:
        _native.transpile(
            """def Button(label: str) -> Component:
    <button>{label}</button>
""",
            "controls.hyper",
        )

    message = str(caught.value)
    assert "keyword-only" in message
    assert "def Button(*, label: str) -> Component:" in message


@pytest.mark.parametrize(
    "signature",
    [
        "label: str, /",
        "*labels",
    ],
)
def test_declared_components_reject_positional_signature_forms(signature):
    with pytest.raises(Exception) as caught:
        _native.transpile(
            f"""def Button({signature}) -> Component:
    <button>Save</button>
""",
            "controls.hyper",
        )

    message = str(caught.value)
    assert "component" in message.lower()
    assert "keyword-only" in message.lower()


@pytest.mark.parametrize(
    "actions_fill",
    [
        "<button {...actions}>Delete</button>",
        """<{...actions}>
        <button>Delete</button>
    </{...actions}>""",
    ],
    ids=["single-element", "explicit-wrapper"],
)
def test_hyper_component_named_slot_composition(
    tmp_path, monkeypatch, actions_fill
):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "app" / "components" / "Card.hyper",
        """title: str
---
<article>
    <h2>{title}</h2>
    <main>{...}</main>
    <footer>
        <{...actions}>
            <button>Cancel</button>
        </{...actions}>
    </footer>
</article>
""",
    )
    write(
        tmp_path / "app" / "pages" / "Confirm.hyper",
        f"""from app.components import Card
---
<{{Card}} title="Delete item">
    <p>This cannot be undone.</p>
    {actions_fill}
</{{Card}}>
""",
    )

    from app.pages import Confirm

    assert Confirm().render() == (
        "<article><h2>Delete item</h2><main><p>This cannot be undone.</p></main>"
        "<footer><button>Delete</button></footer></article>"
    )
    assert "actions" not in inspect.signature(Confirm).parameters


def test_component_tags_pass_declared_props_by_keyword(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "app" / "components" / "controls.hyper",
        """def Button(*, label: str) -> Component:
    <button>{label}</button>
""",
    )
    write(
        tmp_path / "app" / "pages" / "Page.hyper",
        """from app.components.controls import Button
---
<{Button} label="Save" />
""",
    )

    from app.pages import Page

    assert Page().render() == "<button>Save</button>"
