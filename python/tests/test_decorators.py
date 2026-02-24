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


def test_html_result_iter():
    @html
    def MyComponent():
        yield "<p>one</p>"
        yield "<p>two</p>"

    chunks = list(MyComponent())
    assert chunks == ["<p>one</p>", "<p>two</p>"]


def test_html_result_yield_from():
    @html
    def Inner(*, text: str = ""):
        yield f"<span>{text}</span>"

    @html
    def Outer():
        yield "<div>"
        yield from Inner(text="hello")
        yield "</div>"

    assert str(Outer()) == "<div><span>hello</span></div>"


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

