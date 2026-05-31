"""Tests for using Hyper components with Sanic.

A component is a ``str`` subclass; return it via ``response.html``. ``.stream()``
feeds a ``ResponseStream``. No wrappers, no integration code.

The snippets here mirror the Sanic block in docs/design/integrations.md.
"""

from __future__ import annotations

from sanic import Sanic, response
from sanic.response import ResponseStream

from hyper import escape, html


@html
def Greeting(*, name: str):
    yield "<h1>Hello "
    yield escape(name)  # a component escapes its own inputs
    yield "</h1>"


def _app(name: str) -> Sanic:
    Sanic.test_mode = True
    return Sanic(name)


def test_return_component_is_html():
    app = _app("hyper_return")

    @app.get("/")
    async def index(request):
        return response.html(Greeting(name="Ada"))

    _, r = app.test_client.get("/")

    assert r.status == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<h1>Hello Ada</h1>"


def test_user_input_escaped_in_component():
    app = _app("hyper_escape")

    @app.get("/")
    async def index(request):
        return response.html(Greeting(name="<script>"))  # stands in for user input

    _, r = app.test_client.get("/")

    assert "<script>" not in r.text       # the component escaped the input
    assert "&lt;script&gt;" in r.text
    assert "<h1>Hello" in r.text          # but Sanic did not escape its tags


def test_stream_method_feeds_a_stream_response():
    app = _app("hyper_stream")

    @app.get("/stream")
    async def stream(request):
        async def body(res):
            for chunk in Greeting.stream(name="Ada"):
                await res.write(chunk)

        return ResponseStream(body, content_type="text/html")

    _, r = app.test_client.get("/stream")

    assert r.status == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<h1>Hello Ada</h1>"
