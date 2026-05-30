import inspect
import pytest
from hyper.decorators import html


def test_html_preserves_function_name():
    @html
    def MyComponent(*, title: str = ""):
        yield f"<h1>{title}</h1>"

    assert MyComponent.__name__ == "MyComponent"


def test_html_preserves_signature():
    @html
    def MyComponent(*, title: str = "", count: int = 0):
        yield f"<h1>{title}</h1>"

    sig = inspect.signature(MyComponent)
    assert "title" in sig.parameters
    assert "count" in sig.parameters


def test_html_result_str():
    @html
    def MyComponent(*, title: str = "World"):
        yield f"<h1>{title}</h1>"

    assert str(MyComponent(title="Hello")) == "<h1>Hello</h1>"


def test_html_result_is_a_str_subclass():
    @html
    def MyComponent(*, title: str = "World"):
        yield f"<h1>{title}</h1>"

    result = MyComponent(title="Hello")
    assert isinstance(result, str)
    assert result == "<h1>Hello</h1>"


def test_html_result_has_markupsafe_html_protocol():
    @html
    def MyComponent():
        yield "<p>safe</p>"

    result = MyComponent()
    assert hasattr(result, "__html__")
    assert result.__html__() == "<p>safe</p>"


def test_stream_yields_chunks_without_materialization():
    @html
    def MyComponent():
        yield "<p>one</p>"
        yield "<p>two</p>"

    chunks = list(MyComponent.stream())
    assert chunks == ["<p>one</p>", "<p>two</p>"]


def test_yield_from_composition_still_produces_correct_html():
    @html
    def Inner(*, text: str = ""):
        yield f"<span>{text}</span>"

    @html
    def Outer():
        yield "<div>"
        yield from Inner(text="hello")
        yield "</div>"

    # `Inner(...)` is now a str — `yield from` on it yields characters.
    # The outer wrapper "".join still produces correct HTML.
    assert Outer() == "<div><span>hello</span></div>"


def test_html_rejects_positional_args():
    @html
    def MyComponent(*, title: str = ""):
        yield f"<h1>{title}</h1>"

    with pytest.raises(TypeError):
        MyComponent("oops")


def test_html_is_not_a_class():
    @html
    def MyComponent():
        yield "<p>hi</p>"

    assert not isinstance(MyComponent, type)
    assert callable(MyComponent)

