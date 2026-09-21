from __future__ import annotations

import asyncio
import inspect

import pytest


def test_component_call_binds_props_without_rendering():
    from hyper import Component, component

    calls = []

    @component
    def Greeting(*, name: str = "World"):
        calls.append(name)
        yield f"<p>Hello {name}</p>"

    greeting = Greeting(name="Ada")

    assert isinstance(Greeting, Component)
    assert isinstance(greeting, Component)
    assert callable(Greeting)
    assert Greeting.__name__ == "Greeting"
    assert calls == []
    assert inspect.signature(Greeting) == inspect.signature(Greeting.__wrapped__)
    assert inspect.unwrap(Greeting) is Greeting.__wrapped__


def test_sync_component_rendering_is_safe_and_runs_each_time():
    from hyper import HtmlResult, component

    calls = []

    @component
    def Greeting(*, name: str):
        calls.append(name)
        yield f"<p>Hello {name}</p>"

    greeting = Greeting(name="Ada")

    first = greeting.render()
    second = greeting.render()

    assert isinstance(first, HtmlResult)
    assert first == second == "<p>Hello Ada</p>"
    assert first.__html__() == first
    assert calls == ["Ada", "Ada"]


def test_sync_string_conversion_renders_but_repr_does_not():
    from hyper import component

    calls = []

    @component
    def Greeting(*, name: str):
        calls.append(name)
        yield f"<p>Hello {name}</p>"

    greeting = Greeting(name="Ada")

    assert "Greeting" in repr(greeting)
    assert calls == []
    assert str(greeting) == "<p>Hello Ada</p>"
    assert f"{greeting}" == "<p>Hello Ada</p>"
    assert calls == ["Ada", "Ada"]


def test_render_stream_returns_a_fresh_iterator_and_reruns_the_template():
    from hyper import component

    calls = []

    @component
    def Greeting(*, name: str):
        calls.append(name)
        yield "<p>"
        yield name
        yield "</p>"

    greeting = Greeting(name="Ada")
    first = greeting.render(stream=True)
    second = greeting.render(stream=True)

    assert calls == []
    assert list(first) == ["<p>", "Ada", "</p>"]
    assert list(second) == ["<p>", "Ada", "</p>"]
    assert calls == ["Ada", "Ada"]
    with pytest.raises(TypeError):
        greeting.render(True)


def test_async_component_uses_the_same_component_and_render_api():
    from hyper import Component, component

    calls = []

    @component
    async def Greeting(*, name: str):
        calls.append(name)
        yield "<p>"
        await asyncio.sleep(0)
        yield name
        yield "</p>"

    greeting = Greeting(name="Ada")

    async def render():
        html = await greeting.render()
        chunks = [chunk async for chunk in greeting.render(stream=True)]
        return html, chunks

    html, chunks = asyncio.run(render())

    assert isinstance(greeting, Component)
    assert html == "<p>Ada</p>"
    assert chunks == ["<p>", "Ada", "</p>"]
    assert calls == ["Ada", "Ada"]


def test_async_component_rejects_string_conversion_without_rendering():
    from hyper import component

    calls = []

    @component
    async def Greeting():
        calls.append("rendered")
        yield "<p>Hello</p>"

    greeting = Greeting()

    assert "Greeting" in repr(greeting)
    with pytest.raises(TypeError, match=r"await .*\.render\(\)"):
        str(greeting)
    assert calls == []


def test_component_decorator_rejects_positional_parameters():
    from hyper import component

    with pytest.raises(TypeError, match="keyword-only"):

        @component
        def Greeting(name: str):
            yield f"<p>Hello {name}</p>"


def test_subcomponents_are_named_read_only_component_attributes():
    from hyper import component

    @component
    def Logo():
        yield "<strong>Hyper</strong>"

    @component(subcomponents=[Logo])
    def Header():
        yield from Logo().render(stream=True)

    assert Header.Logo is Logo
    assert Header().render() == "<strong>Hyper</strong>"
    assert "Logo" in dir(Header)
    with pytest.raises(AttributeError):
        Header.Logo = Header
    with pytest.raises(AttributeError, match="Header has no component 'Missing'"):
        Header.Missing
