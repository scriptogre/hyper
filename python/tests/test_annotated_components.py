"""Target contract for annotated templates and ordinary component factories."""

from __future__ import annotations

import asyncio
import importlib
import inspect

import pytest

pytest.importorskip("hyperhtml._native")

from hyperhtml import _native


@pytest.fixture
def compile_library():
    def compile_source(source: str):
        imports = "from hyper import Component, subcomponent\n"
        generated = _native.transpile(imports + source, "library.hyper")
        namespace = {}
        exec(compile(generated, "library.hyper", "exec", dont_inherit=True), namespace)
        return namespace

    return compile_source


def test_annotated_factory_preserves_returned_identity(compile_library):
    library = compile_library("""
def choose(*, value: Component) -> Component:
    return value
""")
    value = object()

    assert inspect.isfunction(library["choose"])
    assert library["choose"](value=value) is value


def test_async_factory_remains_an_ordinary_coroutine_function(compile_library):
    library = compile_library("""
async def choose(*, value: Component) -> Component:
    return value
""")
    value = object()

    assert inspect.iscoroutinefunction(library["choose"])
    assert asyncio.run(library["choose"](value=value)) is value


@pytest.mark.parametrize(
    ("body", "expected"),
    [
        ("    <h1>{name}</h1>\n", "<h1>&lt;Ada&gt;</h1>"),
        ("    if name:\n        <h1>{name}</h1>\n    end\n", "<h1>&lt;Ada&gt;</h1>"),
    ],
    ids=["element", "conditional-markup"],
)
def test_template_returns_a_lazy_renderable_instance(compile_library, body, expected):
    library = compile_library(
        'calls = []\ndef Greeting(*, name: str) -> Component:\n'
        '    calls.append(name)\n' + body + 'end\n'
    )

    greeting = library["Greeting"](name="<Ada>")

    assert isinstance(greeting, importlib.import_module("hyper").Component)
    assert library["calls"] == []
    assert greeting.render() == expected
    assert library["calls"] == ["<Ada>"]


def test_factory_can_return_an_annotated_template_instance(compile_library):
    library = compile_library("""
def Heading(*, text: str) -> Component:
    <h1>{text}</h1>
end

def Greeting(*, name: str) -> Component:
    return Heading(text=f"Hello, {name}")
end
""")

    assert inspect.isfunction(library["Greeting"])
    assert library["Greeting"](name="Ada").render() == "<h1>Hello, Ada</h1>"


def test_component_call_counts_as_template_output(compile_library):
    library = compile_library("""
def Heading(*, text: str) -> Component:
    <h1>{text}</h1>
end

def Greeting(*, name: str) -> Component:
    <{Heading} text={name} />
end
""")

    assert library["Greeting"](name="Ada").render() == "<h1>Ada</h1>"


def test_nested_template_does_not_turn_its_factory_into_a_template(compile_library):
    library = compile_library("""
def Greeting(*, name: str) -> Component:
    def Heading() -> Component:
        <h1>{name}</h1>
    end
    return Heading()
end
""")

    assert inspect.isfunction(library["Greeting"])
    assert library["Greeting"](name="Ada").render() == "<h1>Ada</h1>"
    assert library["Greeting"](name="Lin").render() == "<h1>Lin</h1>"
    assert not hasattr(library["Greeting"], "Heading")


def test_await_makes_an_annotated_template_async(compile_library):
    library = compile_library("""
async def load_name():
    await asyncio.sleep(0)
    return "Ada"
end

def Greeting() -> Component:
    name = await load_name()
    <h1>{name}</h1>
end
""")
    library["asyncio"] = asyncio

    greeting = library["Greeting"]()

    assert asyncio.run(greeting.render()) == "<h1>Ada</h1>"

    async def stream():
        return [chunk async for chunk in greeting.render(stream=True)]

    assert asyncio.run(stream()) == ["<h1>Ada</h1>"]


def test_subcomponent_exports_an_independent_definition(compile_library):
    library = compile_library("""
def Page(*, title: str) -> Component:
    @subcomponent
    def Header(*, title: str) -> Component:
        <h1>{title}</h1>
    end
    <{Header} title={title} />
end
""")
    page = library["Page"]

    assert page.Header(title="Other").render() == "<h1>Other</h1>"
    assert page(title="Home").render() == "<h1>Home</h1>"
    assert page.Header(title="Again").render() == "<h1>Again</h1>"


def test_python_import_namespace():
    runtime = importlib.import_module("hyper")

    assert inspect.isclass(runtime.Component)
    assert callable(runtime.subcomponent)
    assert runtime.escape("<Ada>") == "&lt;Ada&gt;"
