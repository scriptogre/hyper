from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import ModuleType

import pytest

pytest.importorskip("hyperhtml._native")

from hyperhtml import _autohook


@pytest.fixture(autouse=True)
def hyper_imports():
    _autohook._uninstall_finder()
    _autohook._install_finder()
    yield
    _autohook._uninstall_finder()
    for name in list(sys.modules):
        if name in {"Page", "forms"} or name == "app" or name.startswith("app."):
            sys.modules.pop(name, None)


def write(path: Path, source: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source)


def test_separator_selects_empty_implicit_component(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "app" / "pages" / "Empty.hyper", "---\n")

    from app.pages import Empty
    from hyperhtml import Component

    assert isinstance(Empty, Component)
    assert Empty() == ""


def test_top_level_output_selects_implicit_component(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "app" / "pages" / "Home.hyper", "<h1>Home</h1>\n")

    from app.pages import Home
    from hyperhtml import Component

    assert isinstance(Home, Component)
    assert Home() == "<h1>Home</h1>"


def test_declaration_only_file_is_normal_library_module(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "app" / "components" / "forms.hyper",
        """DEFAULT_LABEL = "Save"

component Button(*, label: str = DEFAULT_LABEL):
    <button>{label}</button>
end
""",
    )

    forms = importlib.import_module("app.components.forms")

    assert isinstance(forms, ModuleType)
    assert not callable(forms)
    assert forms.DEFAULT_LABEL == "Save"
    assert forms.Button() == "<button>Save</button>"


def test_root_level_library_is_supported(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(
        tmp_path / "forms.hyper",
        """component Button(*, label: str):
    <button>{label}</button>
end
""",
    )

    from forms import Button

    assert Button(label="Save") == "<button>Save</button>"


def test_lowercase_implicit_component_imports_as_package_attribute(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "app" / "templates" / "index.hyper", "<h1>Home</h1>\n")

    from app.templates import index
    from hyperhtml import Component

    assert isinstance(index, Component)
    assert index.__name__ == "Index"
    assert index() == "<h1>Home</h1>"


def test_root_level_implicit_component_is_rejected(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "Page.hyper", "<h1>Page</h1>\n")

    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("Page")


def test_python_module_wins_over_hyper_file(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "app" / "pages" / "Home.py", 'kind = "python"\n')
    write(tmp_path / "app" / "pages" / "Home.hyper", "<h1>Hyper</h1>\n")

    from app.pages import Home

    assert isinstance(Home, ModuleType)
    assert Home.kind == "python"


def test_explicit_package_attribute_wins_over_hyper_file(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "app" / "pages" / "__init__.py", 'Home = "package"\n')
    write(tmp_path / "app" / "pages" / "Home.hyper", "<h1>Hyper</h1>\n")

    from app.pages import Home

    assert Home == "package"


def test_implicit_component_never_becomes_submodule(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write(tmp_path / "app" / "pages" / "Home.hyper", "<h1>Home</h1>\n")

    import app.pages as pages
    from app.pages import Home

    with pytest.raises(ModuleNotFoundError):
        importlib.import_module("app.pages.Home")

    assert pages.Home is Home
    assert Home() == "<h1>Home</h1>"
