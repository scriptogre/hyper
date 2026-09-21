"""Tests for using Hyper components with Litestar.

A handler renders a component with ``media_type=MediaType.HTML``.
``render(stream=True)`` feeds a Litestar ``Stream`` response.

The snippets here mirror the Litestar block in docs/design/integrations.md.
"""

from __future__ import annotations

from litestar import MediaType, get
from litestar.response import Stream
from litestar.testing import create_test_client

from hyper import component, escape


@component
def Greeting(*, name: str):
    yield "<h1>Hello "
    yield escape(name)  # a component escapes its own inputs
    yield "</h1>"


def test_return_component_is_html():
    @get("/", media_type=MediaType.HTML)
    async def index() -> str:
        return Greeting(name="Ada").render()

    with create_test_client([index]) as client:
        r = client.get("/")

    assert r.status_code == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<h1>Hello Ada</h1>"


def test_user_input_escaped_in_component():
    @get("/", media_type=MediaType.HTML)
    async def index() -> str:
        return Greeting(name="<script>").render()  # stands in for user input

    with create_test_client([index]) as client:
        r = client.get("/")

    assert "<script>" not in r.text  # the component escaped the input
    assert "&lt;script&gt;" in r.text
    assert "<h1>Hello" in r.text  # but Litestar did not escape its tags


def test_stream_method_feeds_a_stream_response():
    @get("/stream", media_type=MediaType.HTML)
    async def stream() -> Stream:
        return Stream(Greeting(name="Ada").render(stream=True))

    with create_test_client([stream]) as client:
        r = client.get("/stream")

    assert r.status_code == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<h1>Hello Ada</h1>"
