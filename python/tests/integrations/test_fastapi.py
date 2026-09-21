"""Tests for using Hyper components with FastAPI.

A route renders a component inside an HTML response. ``render(stream=True)``
feeds a ``StreamingResponse``.

The snippets here mirror the FastAPI block in docs/design/integrations.md.
"""

from __future__ import annotations

from fastapi import FastAPI
from fastapi.responses import HTMLResponse, StreamingResponse
from fastapi.testclient import TestClient

from hyper import component, escape


@component
def Greeting(*, name: str):
    yield "<h1>Hello "
    yield escape(name)  # a component escapes its own inputs
    yield "</h1>"


def test_return_component_with_per_route_html_response():
    app = FastAPI()

    @app.get("/", response_class=HTMLResponse)
    def index():
        return Greeting(name="Ada").render()

    r = TestClient(app).get("/")

    assert r.status_code == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<h1>Hello Ada</h1>"


def test_return_component_with_default_response_class():
    """The app-level default_response_class pattern, even cleaner."""
    app = FastAPI(default_response_class=HTMLResponse)

    @app.get("/")
    def index():
        return Greeting(name="Ada").render()

    r = TestClient(app).get("/")

    assert r.status_code == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<h1>Hello Ada</h1>"


def test_user_input_escaped_in_component():
    app = FastAPI(default_response_class=HTMLResponse)

    @app.get("/{name}")
    def show(name: str):
        return Greeting(name=name).render()

    r = TestClient(app).get("/<script>")

    assert "<script>" not in r.text
    assert "&lt;script&gt;" in r.text


def test_streaming_response_with_stream_method():
    app = FastAPI()

    @app.get("/stream")
    def stream():
        greeting = Greeting(name="Ada")
        return StreamingResponse(greeting.render(stream=True), media_type="text/html")

    r = TestClient(app).get("/stream")

    assert r.status_code == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<h1>Hello Ada</h1>"


def test_returning_html_response_directly_still_works():
    """The explicit form, for users who want it."""
    app = FastAPI()

    @app.get("/")
    def index():
        return HTMLResponse(Greeting(name="Ada").render())

    r = TestClient(app).get("/")

    assert r.status_code == 200
    assert r.text == "<h1>Hello Ada</h1>"
