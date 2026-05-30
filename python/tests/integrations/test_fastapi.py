"""Tests for FastAPI integration.

Verifies the goal: write standard FastAPI routes, return Hyper components
directly, no wrappers, no str() calls, and no explicit hyper.integrations.*
import — Hyper auto-registers with FastAPI on first component instantiation.
"""

from __future__ import annotations

import pytest
from fastapi import FastAPI
from fastapi.responses import HTMLResponse, StreamingResponse
from fastapi.testclient import TestClient

from hyper import html


@html
def Sidebar(*, user: str):
    yield f"<aside>{user}</aside>"


@html
def Page(*, title: str):
    yield f"<html><body><h1>{title}</h1></body></html>"


def test_return_component_with_per_route_html_response():
    app = FastAPI()

    @app.get("/", response_class=HTMLResponse)
    def index():
        return Sidebar(user="Ada")

    client = TestClient(app)
    r = client.get("/")

    assert r.status_code == 200
    assert r.headers["content-type"].startswith("text/html")
    assert r.text == "<aside>Ada</aside>"


def test_return_component_with_default_response_class():
    """The app-level default_response_class pattern — even cleaner."""
    app = FastAPI(default_response_class=HTMLResponse)

    @app.get("/")
    def index():
        return Page(title="Welcome")

    client = TestClient(app)
    r = client.get("/")

    assert r.status_code == 200
    assert r.headers["content-type"].startswith("text/html")
    assert "<h1>Welcome</h1>" in r.text


def test_user_input_escaped_in_component(components_dir):
    """Real compiled components escape their inputs — verify end-to-end through FastAPI."""
    import sys
    sys.path.insert(0, str(components_dir.parent.parent))
    try:
        from fixtures.components.Greeting import Greeting
    finally:
        sys.path.pop(0)

    app = FastAPI(default_response_class=HTMLResponse)

    @app.get("/{name}")
    def show(name: str):
        return Greeting(name=name)

    client = TestClient(app)
    r = client.get("/<script>")

    assert "<script>" not in r.text
    assert "&lt;script&gt;" in r.text


def test_streaming_response_consumes_component_chunks():
    """StreamingResponse already works because HtmlResult is iterable — no glue needed."""

    @html
    def Many():
        for i in range(3):
            yield f"<p>{i}</p>"

    app = FastAPI()

    @app.get("/")
    def index():
        return StreamingResponse(Many(), media_type="text/html")

    client = TestClient(app)
    r = client.get("/")

    assert r.status_code == 200
    assert r.text == "<p>0</p><p>1</p><p>2</p>"


def test_returning_html_response_directly_still_works():
    """Users who want the explicit form should still be able to use it."""
    app = FastAPI()

    @app.get("/")
    def index():
        # HtmlResult has an .encode() method? No — but str(HtmlResult) is the
        # documented path for explicit HTMLResponse use. This test pins that
        # behavior.
        return HTMLResponse(str(Sidebar(user="Ada")))

    client = TestClient(app)
    r = client.get("/")

    assert r.status_code == 200
    assert r.text == "<aside>Ada</aside>"
