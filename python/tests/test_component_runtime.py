from __future__ import annotations

import asyncio
import inspect

import pytest


def test_component_decorator_returns_component_with_python_metadata():
    from hyperhtml import Component, component

    @component
    def Greeting(*, name: str = "World"):
        yield f"<p>Hello {name}</p>"

    assert isinstance(Greeting, Component)
    assert Greeting(name="Ada") == "<p>Hello Ada</p>"
    assert Greeting.__name__ == "Greeting"
    assert Greeting.stream is Greeting.__wrapped__
    assert inspect.signature(Greeting) == inspect.signature(Greeting.__wrapped__)
    assert Greeting(name="Ada").__html__() == "<p>Hello Ada</p>"


def test_component_decorator_rejects_positional_parameters():
    from hyperhtml import component

    with pytest.raises(TypeError, match="keyword-only"):

        @component
        def Greeting(name: str):
            yield f"<p>Hello {name}</p>"


def test_subcomponents_are_named_read_only_component_attributes():
    from hyperhtml import component

    @component
    def Logo():
        yield "<strong>Hyper</strong>"

    @component(subcomponents=[Logo])
    def Header():
        yield from Logo.stream()

    assert Header.Logo is Logo
    assert Header() == "<strong>Hyper</strong>"
    assert "Logo" in dir(Header)
    with pytest.raises(AttributeError):
        Header.Logo = Header
    with pytest.raises(AttributeError, match="Header has no component 'Missing'"):
        Header.Missing


def test_stream_returns_fresh_validated_chunk_iterators():
    from hyperhtml import component

    @component
    def Greeting(*, name: str):
        yield "<p>"
        yield name
        yield "</p>"

    first = Greeting.stream(name="Ada")
    second = Greeting.stream(name="Lin")

    assert list(first) == ["<p>", "Ada", "</p>"]
    assert list(second) == ["<p>", "Lin", "</p>"]
    with pytest.raises(TypeError):
        Greeting.stream()


def test_component_decorator_supports_async_call_and_stream_apis():
    from hyperhtml import Component, component

    @component
    async def Greeting(*, name: str):
        yield "<p>"
        yield name
        yield "</p>"

    async def render():
        output = await Greeting(name="Ada")
        chunks = [chunk async for chunk in Greeting.stream(name="Lin")]
        return output, chunks

    assert isinstance(Greeting, Component)
    assert asyncio.run(render()) == (
        "<p>Ada</p>",
        ["<p>", "Lin", "</p>"],
    )
    with pytest.raises(TypeError):
        Greeting.stream()
