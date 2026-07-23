import inspect

import pytest

from hyperhtml.decorators import component


def test_component_preserves_function_name():
    @component
    def MyComponent(*, title: str = ""):
        yield f"<h1>{title}</h1>"

    assert MyComponent.__name__ == "MyComponent"


def test_component_preserves_signature():
    @component
    def MyComponent(*, title: str = "", count: int = 0):
        yield f"<h1>{title}</h1>"

    signature = inspect.signature(MyComponent)
    assert "title" in signature.parameters
    assert "count" in signature.parameters


def test_component_result_str():
    @component
    def MyComponent(*, title: str = "World"):
        yield f"<h1>{title}</h1>"

    assert str(MyComponent(title="Hello")) == "<h1>Hello</h1>"


def test_component_result_is_a_str_subclass():
    @component
    def MyComponent(*, title: str = "World"):
        yield f"<h1>{title}</h1>"

    result = MyComponent(title="Hello")
    assert isinstance(result, str)
    assert result == "<h1>Hello</h1>"


def test_component_result_has_markupsafe_html_protocol():
    @component
    def MyComponent():
        yield "<p>safe</p>"

    result = MyComponent()
    assert hasattr(result, "__html__")
    assert result.__html__() == "<p>safe</p>"


def test_stream_yields_chunks_without_materialization():
    @component
    def MyComponent():
        yield "<p>one</p>"
        yield "<p>two</p>"

    assert list(MyComponent.stream()) == ["<p>one</p>", "<p>two</p>"]


def test_stream_composition_preserves_chunks():
    @component
    def Inner(*, text: str = ""):
        yield f"<span>{text}</span>"

    @component
    def Outer():
        yield "<div>"
        yield from Inner.stream(text="hello")
        yield "</div>"

    assert Outer() == "<div><span>hello</span></div>"
    assert list(Outer.stream()) == ["<div>", "<span>hello</span>", "</div>"]


def test_component_rejects_positional_args():
    @component
    def MyComponent(*, title: str = ""):
        yield f"<h1>{title}</h1>"

    with pytest.raises(TypeError):
        MyComponent("oops")


def test_component_is_a_callable_object():
    @component
    def MyComponent():
        yield "<p>hi</p>"

    assert not isinstance(MyComponent, type)
    assert callable(MyComponent)
