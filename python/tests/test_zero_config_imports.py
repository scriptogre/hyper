from __future__ import annotations

import sys
from pathlib import Path

import pytest

pytest.importorskip("hyperhtml._native")

from hyperhtml import _autohook


@pytest.fixture(autouse=True)
def hyper_imports(monkeypatch):
    _autohook._uninstall_finder()
    _autohook._install_finder()
    yield
    _autohook._uninstall_finder()
    for name in list(sys.modules):
        if name == "app" or name.startswith("app."):
            sys.modules.pop(name, None)


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def test_short_import_loads_component_from_hyper_file(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "app" / "components" / "Greeting.hyper",
        """name: str
---
<p>Hello {name}</p>
""",
    )

    from app.components import Greeting

    assert Greeting(name="Ada") == "<p>Hello Ada</p>"
    assert not list(tmp_path.rglob("*.py"))


def test_init_attribute_shadows_short_hyper_lookup(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "app" / "components" / "__init__.py", 'Greeting = "shadow"\n')
    write(
        tmp_path / "app" / "components" / "Greeting.hyper",
        """name: str
---
<p>Hello {name}</p>
""",
    )

    from app.components import Greeting

    assert Greeting == "shadow"


def test_component_imports_another_component(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "app" / "components" / "Greeting.hyper",
        """name: str
---
<span>Hello {name}</span>
""",
    )
    write(
        tmp_path / "app" / "components" / "Card.hyper",
        """from . import Greeting
name: str
---
<div>
    <{Greeting} name={name} />
</div>
""",
    )

    from app.components import Card

    assert Card(name="Ada") == "<div><span>Hello Ada</span></div>"
