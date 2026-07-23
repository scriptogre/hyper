from __future__ import annotations

import asyncio
import inspect
import sys
from pathlib import Path

import pytest

pytest.importorskip("hyperhtml._native")
pytestmark = pytest.mark.skip(reason="@render_here is coming soon")

from hyperhtml import _autohook


@pytest.fixture(autouse=True)
def hyper_imports():
    _autohook._uninstall_finder()
    _autohook._install_finder()
    yield
    _autohook._uninstall_finder()
    for name in list(sys.modules):
        if name == "app" or name.startswith("app."):
            sys.modules.pop(name, None)


def write_page(tmp_path: Path, source: str) -> None:
    package = tmp_path / "app" / "pages"
    package.mkdir(parents=True)
    (package / "Page.hyper").write_text(source)


def test_subcomponent_is_exported_rendered_here_and_streamable(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """title: str
---
<main>
    @render_here
    component Header(*, title: str):
        <header>{title}</header>
    end
</main>
""",
    )

    from app.pages import Page
    from hyperhtml import Component

    assert isinstance(Page, Component)
    assert isinstance(Page.Header, Component)
    assert Page.Header(title="Other") == "<header>Other</header>"
    assert list(Page.Header.stream(title="Other")) == ["<header>Other</header>"]
    assert Page(title="Home") == "<main><header>Home</header></main>"
    assert list(Page.stream(title="Home")) == [
        "<main>",
        "<header>Home</header>",
        "</main>",
    ]
    assert Page.__name__ == "Page"
    assert Page.Header.__name__ == "Header"
    assert list(inspect.signature(Page).parameters) == ["title"]
    assert list(inspect.signature(Page.Header).parameters) == ["title"]
    with pytest.raises(AttributeError):
        Page.Header = Page
    assert not list(tmp_path.rglob("*.py"))


def test_plain_subcomponent_declaration_exports_without_rendering(
    tmp_path, monkeypatch
):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """---
<main>
    component Footer():
        <footer>Outside</footer>
    end
</main>
""",
    )

    from app.pages import Page

    assert Page() == "<main></main>"
    assert Page.Footer() == "<footer>Outside</footer>"
    assert list(Page.Footer.stream()) == ["<footer>Outside</footer>"]


def test_render_here_arguments_override_name_binding_and_keep_defaults(
    tmp_path, monkeypatch
):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """page_title: str
user: str
---
@render_here(title=page_title)
component Header(*, title: str, user: str, suffix: str = "!"):
    <header>{title}, {user}{suffix}</header>
end
""",
    )

    from app.pages import Page

    assert Page(page_title="Home", user="Ada") == "<header>Home, Ada!</header>"
    assert (
        Page.Header(title="Other", user="Lin", suffix=".")
        == "<header>Other, Lin.</header>"
    )


def test_matching_locals_loop_variables_and_optional_parameters_bind_here(
    tmp_path, monkeypatch
):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """title: str
items: list[str]
suffix: str
---
heading = title.upper()

@render_here
component Heading(*, heading: str):
    <h1>{heading}</h1>
end

for item in items:
    @render_here
    component Row(*, item: str, suffix: str = "!"):
        <p>{item}{suffix}</p>
    end
end
""",
    )

    from app.pages import Page

    assert Page(title="Home", items=["A", "B"], suffix=".") == (
        "<h1>HOME</h1><p>A.</p><p>B.</p>"
    )
    assert Page.Heading(heading="Other") == "<h1>Other</h1>"
    assert Page.Row(item="C") == "<p>C!</p>"


def test_parent_render_names_must_be_declared_as_component_parameters(
    tmp_path, monkeypatch
):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """title: str
---
@render_here
component Header():
    <h1>{title}</h1>
end
""",
    )

    with pytest.raises(Exception) as caught:
        from app.pages import Page  # noqa: F401

    message = str(caught.value)
    assert "Header" in message
    assert "title" in message
    assert "parameter" in message.lower()


def test_render_here_rejects_missing_required_values(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """---
@render_here
component Header(*, title: str):
    <h1>{title}</h1>
end
""",
    )

    with pytest.raises(Exception) as caught:
        from app.pages import Page  # noqa: F401

    message = str(caught.value)
    assert "Header" in message
    assert "title" in message
    assert "required" in message.lower()


def test_render_here_rejects_arguments_absent_from_signature(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """page_title: str
---
@render_here(title=page_title)
component Header():
    <h1>Home</h1>
end
""",
    )

    with pytest.raises(Exception) as caught:
        from app.pages import Page  # noqa: F401

    message = str(caught.value)
    assert "Header" in message
    assert "title" in message
    assert "parameter" in message.lower()


def test_control_flow_controls_rendering_not_export_creation(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """show: bool
---
if show:
    @render_here
    component Header():
        <header>Visible</header>
    end
end
""",
    )

    from app.pages import Page

    assert Page.Header() == "<header>Visible</header>"
    assert Page(show=False) == ""
    assert Page(show=True) == "<header>Visible</header>"


def test_nested_subcomponents_create_nested_component_namespaces(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """---
@render_here
component Header():
    component Logo():
        <strong>Hyper</strong>
    end

    <header><{Logo} /></header>
end
""",
    )

    from app.pages import Page
    from hyperhtml import Component

    assert isinstance(Page.Header.Logo, Component)
    assert Page() == "<header><strong>Hyper</strong></header>"
    assert Page.Header.Logo() == "<strong>Hyper</strong>"
    assert list(Page.Header.Logo.stream()) == ["<strong>Hyper</strong>"]
    assert "Header" in dir(Page)
    assert "Logo" in dir(Page.Header)
    assert not hasattr(Page, "Page")
    with pytest.raises(AttributeError, match="Page has no component 'Missing'"):
        Page.Missing


def test_async_render_here_makes_the_parent_and_subcomponent_async(
    tmp_path, monkeypatch
):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """from . import pause
name: str
---
@render_here
async component Greeting(*, name: str):
    await pause()
    <p>Hello {name}</p>
end
""",
    )
    (tmp_path / "app" / "pages" / "__init__.py").write_text(
        "import asyncio\n\nasync def pause():\n    await asyncio.sleep(0)\n"
    )

    from app.pages import Page

    async def render():
        page = await Page(name="Ada")
        child = await Page.Greeting(name="Lin")
        chunks = [chunk async for chunk in Page.stream(name="Bea")]
        return page, child, chunks

    assert asyncio.run(render()) == (
        "<p>Hello Ada</p>",
        "<p>Hello Lin</p>",
        ["<p>Hello Bea</p>"],
    )


def test_component_api_names_cannot_be_reused_as_exports(tmp_path, monkeypatch):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """---
component stream():
    <p>Conflict</p>
end
""",
    )

    with pytest.raises(Exception) as caught:
        from app.pages import Page  # noqa: F401

    assert "stream" in str(caught.value)
    assert "reserved" in str(caught.value).lower()


def test_render_here_uses_slot_fallback_and_explicit_call_accepts_content(
    tmp_path, monkeypatch
):
    monkeypatch.syspath_prepend(str(tmp_path))
    write_page(
        tmp_path,
        """---
@render_here
component FallbackPanel():
    <section>
        <{...}>
            <p>Fallback</p>
        </{...}>
    </section>
end

component CustomPanel():
    <section>{...}</section>
end

component Layout():
    <header>{...header}</header>
    <main>{...}</main>
end

<{CustomPanel}>
    <p>Custom</p>
</{CustomPanel}>
""",
    )

    from app.pages import Page

    assert (
        Page() == "<section><p>Fallback</p></section><section><p>Custom</p></section>"
    )
    assert Page.FallbackPanel() == "<section><p>Fallback</p></section>"

    fallback = Page.FallbackPanel()
    assert Page.CustomPanel(content=fallback) == (
        "<section><section><p>Fallback</p></section></section>"
    )
    assert Page.Layout(content=fallback, header=fallback) == (
        "<header><section><p>Fallback</p></section></header>"
        "<main><section><p>Fallback</p></section></main>"
    )

    custom_signature = inspect.signature(Page.CustomPanel)
    layout_signature = inspect.signature(Page.Layout)
    assert custom_signature.parameters["content"].kind is inspect.Parameter.KEYWORD_ONLY
    assert layout_signature.parameters["content"].kind is inspect.Parameter.KEYWORD_ONLY
    assert layout_signature.parameters["header"].kind is inspect.Parameter.KEYWORD_ONLY
