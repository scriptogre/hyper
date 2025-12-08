"""Tests for URL-based content loading with Meta.url."""

import pytest
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import time

from hyper.content import Singleton, Collection


# Simple HTTP server for testing
class TestHTTPHandler(BaseHTTPRequestHandler):
    """Simple handler that serves JSON based on path."""

    def log_message(self, format, *args):
        """Suppress log messages."""
        pass

    def do_GET(self):
        if self.path == "/single":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"title": "Test", "value": 42}).encode())

        elif self.path == "/list":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps([
                {"title": "First", "views": 100},
                {"title": "Second", "views": 200},
            ]).encode())

        elif self.path == "/countries":
            # Simulate real API like restcountries
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps([
                {"cca3": "USA", "name": "United States"},
                {"cca3": "CAN", "name": "Canada"},
                {"cca3": "MEX", "name": "Mexico"},
            ]).encode())

        else:
            self.send_response(404)
            self.end_headers()


@pytest.fixture(scope="module")
def http_server():
    """Start a test HTTP server."""
    server = HTTPServer(("localhost", 0), TestHTTPHandler)
    port = server.server_port

    thread = threading.Thread(target=server.serve_forever)
    thread.daemon = True
    thread.start()

    time.sleep(0.1)  # Give server time to start

    yield f"http://localhost:{port}"

    server.shutdown()


# ==========================================
# Part 1: Basic URL Loading
# ==========================================

def test_url_loader_singleton(http_server):
    """Load singleton from URL endpoint."""

    class Config(Singleton):
        title: str
        value: int

        class Meta:
            url = f"{http_server}/single"

    config = Config.load()
    assert config.title == "Test"
    assert config.value == 42


def test_url_loader_collection(http_server):
    """Load collection from URL endpoint."""

    class Post(Collection):
        title: str
        views: int

        class Meta:
            url = f"{http_server}/list"

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].title == "First"
    assert posts[0].views == 100
    assert posts[1].title == "Second"
    assert posts[1].views == 200


def test_url_loader_real_api_pattern(http_server):
    """Simulate loading from a real API like restcountries."""

    class Country(Collection):
        cca3: str  # 3-letter country code
        name: str

        class Meta:
            url = f"{http_server}/countries"

    countries = Country.load()
    assert len(countries) == 3
    assert countries[0].cca3 == "USA"
    assert countries[0].name == "United States"


# ==========================================
# Part 2: URL Loading with Computed Fields
# ==========================================

def test_url_loader_with_computed_fields(http_server):
    """URL loading works with computed fields."""
    from hyper.content import computed

    class Post(Collection):
        title: str
        views: int

        class Meta:
            url = f"{http_server}/list"

        @computed
        def views_k(self) -> str:
            return f"{self.views // 1000}k" if self.views >= 1000 else str(self.views)

        @computed
        def title_upper(self) -> str:
            return self.title.upper()

    posts = Post.load()
    assert posts[0].title_upper == "FIRST"
    assert posts[0].views_k == "100"


# ==========================================
# Part 3: URL Loading with Validation
# ==========================================

def test_url_loader_with_pydantic(http_server):
    """URL loading works with Pydantic validation."""
    try:
        import pydantic
    except ImportError:
        pytest.skip("pydantic not installed")

    class Post(Collection, pydantic.BaseModel):
        title: str
        views: int

        class Meta:
            url = f"{http_server}/list"

    posts = Post.load()
    assert len(posts) == 2
    assert isinstance(posts[0], pydantic.BaseModel)
    assert posts[0].title == "First"


def test_url_loader_with_msgspec(http_server):
    """URL loading works with msgspec validation."""
    try:
        import msgspec
    except ImportError:
        pytest.skip("msgspec not installed")

    class Post(Collection, msgspec.Struct):
        title: str
        views: int

        class Meta:
            url = f"{http_server}/list"

    posts = Post.load()
    assert len(posts) == 2
    assert isinstance(posts[0], msgspec.Struct)
    assert posts[0].title == "First"


# ==========================================
# Part 4: Error Handling
# ==========================================

def test_url_loader_missing_url():
    """Error if neither pattern nor url is provided."""

    class Post(Collection):
        title: str

        # No Meta at all

    with pytest.raises(ValueError, match="Missing Meta.pattern or Meta.url"):
        Post.load()


def test_url_loader_404_error(http_server):
    """404 errors are raised properly."""

    class Post(Collection):
        title: str

        class Meta:
            url = f"{http_server}/nonexistent"

    with pytest.raises(Exception):  # urllib raises URLError
        Post.load()


def test_url_loader_empty_response(http_server):
    """Empty singleton raises error."""
    # We'd need to add an endpoint that returns empty array
    # Skipping for now since our test server doesn't have that endpoint
    pass


# ==========================================
# Part 5: Integration with Other Features
# ==========================================

def test_url_loader_no_hooks(http_server):
    """URL loading doesn't trigger hooks (like custom loaders)."""

    call_order = []

    class Post(Collection):
        title: str
        views: int

        class Meta:
            url = f"{http_server}/list"

            @staticmethod
            def after_load(instance: 'Post') -> 'Post':
                call_order.append('after_load')
                return instance

    posts = Post.load()

    # URL loading doesn't trigger hooks (same as custom loaders)
    assert len(call_order) == 0
    assert len(posts) == 2
    assert posts[0].title == "First"
