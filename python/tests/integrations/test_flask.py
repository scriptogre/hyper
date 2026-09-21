"""Tests for using Hyper components with Flask.

A Flask view explicitly renders a component. ``render(stream=True)`` returns
a chunk iterator that Flask streams as-is.

The snippets here mirror the Flask block in docs/design/integrations.md.
"""

from __future__ import annotations

from flask import Flask

from hyper import component, escape


@component
def Greeting(*, name: str):
    yield "<h1>Hello "
    yield escape(name)  # a component escapes its own inputs
    yield "</h1>"


def test_return_component_is_html():
    app = Flask(__name__)

    @app.get("/")
    def index():
        return Greeting(name="Ada").render()

    r = app.test_client().get("/")

    assert r.status_code == 200
    assert r.headers["Content-Type"].startswith("text/html")
    assert r.get_data(as_text=True) == "<h1>Hello Ada</h1>"


def test_user_input_escaped_in_component():
    app = Flask(__name__)

    @app.get("/<name>")
    def show(name):
        return Greeting(name=name).render()

    body = app.test_client().get("/<script>").get_data(as_text=True)

    assert "<script>" not in body
    assert "&lt;script&gt;" in body


def test_stream_method_feeds_a_streaming_response():
    app = Flask(__name__)

    @app.get("/stream")
    def stream():
        return Greeting(name="Ada").render(stream=True)

    r = app.test_client().get("/stream")

    assert r.status_code == 200
    assert r.headers["Content-Type"].startswith("text/html")
    assert r.get_data(as_text=True) == "<h1>Hello Ada</h1>"
