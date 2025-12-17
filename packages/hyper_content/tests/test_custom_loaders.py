"""Tests for custom loaders - overriding .load() method."""

import pytest
from pathlib import Path

try:
    import pydantic
    HAS_PYDANTIC = True
except ImportError:
    HAS_PYDANTIC = False
    pydantic = None

from hyper import Singleton, Collection


# ==========================================
# Part 1: Basic Custom Loaders
# ==========================================

def test_custom_loader_collection_simple():
    """Override load() to return instances from custom source."""

    class Post(Collection):
        title: str
        views: int

        @classmethod
        def load(cls) -> list['Post']:
            # Custom data source (could be API, database, etc.)
            data = [
                {"title": "First", "views": 100},
                {"title": "Second", "views": 200},
            ]
            return [cls(**item) for item in data]

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].title == "First"
    assert posts[0].views == 100
    assert posts[1].title == "Second"
    assert posts[1].views == 200


def test_custom_loader_singleton():
    """Override load() for Singleton."""

    class Config(Singleton):
        api_key: str
        debug: bool

        @classmethod
        def load(cls) -> 'Config':
            # Fetch from environment or API
            data = {"api_key": "test123", "debug": True}
            return cls(**data)

    config = Config.load()
    assert config.api_key == "test123"
    assert config.debug is True


# ==========================================
# Part 2: Custom Loaders - No Hooks
# ==========================================

def test_custom_loader_with_computed_fields():
    """When overriding load(), compute fields manually (no hooks)."""

    class Post(Collection):
        title: str
        views: int
        views_doubled: int = 0

        @classmethod
        def load(cls) -> list['Post']:
            data = [
                {"title": "First", "views": 100},
                {"title": "Second", "views": 200},
            ]
            # Compute fields manually when using custom loader
            return [cls(**item, views_doubled=item["views"] * 2) for item in data]

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].views == 100
    assert posts[0].views_doubled == 200
    assert posts[1].views == 200
    assert posts[1].views_doubled == 400


def test_custom_loader_compute_field_in_loop():
    """Compute derived fields in the loading loop."""

    class Article(Collection):
        title: str
        body: str
        word_count: int = 0

        @classmethod
        def load(cls) -> list['Article']:
            data = [
                {"title": "Hello", "body": "This is a test article"},
                {"title": "World", "body": "Foo bar baz"},
            ]
            # Compute word_count inline
            return [
                cls(title=item["title"], body=item["body"], word_count=len(item["body"].split()))
                for item in data
            ]

    articles = Article.load()
    assert articles[0].word_count == 5  # "This is a test article"
    assert articles[1].word_count == 3  # "Foo bar baz"


# ==========================================
# Part 3: Mixing Custom and File-based Loading
# ==========================================

def test_custom_loader_can_call_super(content_dir):
    """Custom loader can extend file-based loading."""
    # Create some files
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text('{"title": "From File", "views": 100}')

    import os
    os.chdir(content_dir)

    class Post(Collection):
        title: str
        views: int

        class Meta:
            pattern = "posts/*.json"

        @classmethod
        def load(cls) -> list['Post']:
            # Load from files
            from_files = super().load()

            # Add custom data
            extra = cls(title="From API", views=999)

            return from_files + [extra]

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].title == "From File"
    assert posts[0].views == 100
    assert posts[1].title == "From API"
    assert posts[1].views == 999


# ==========================================
# Part 4: Simulating Real-World API Usage
# ==========================================

def test_custom_loader_simulated_api():
    """Simulate loading from REST API."""

    # Simulate API response
    def fake_api_call():
        return {
            "data": [
                {"id": 1, "title": "API Post 1", "published": True},
                {"id": 2, "title": "API Post 2", "published": False},
            ]
        }

    class BlogPost(Collection):
        id: int
        title: str
        published: bool

        @classmethod
        def load(cls) -> list['BlogPost']:
            response = fake_api_call()
            return [cls(**item) for item in response["data"]]

    posts = BlogPost.load()
    assert len(posts) == 2
    assert posts[0].id == 1
    assert posts[0].title == "API Post 1"
    assert posts[1].published is False


def test_custom_loader_with_filtering():
    """Custom loader can filter data before returning."""

    class Post(Collection):
        title: str
        draft: bool

        @classmethod
        def load(cls) -> list['Post']:
            # Fetch all, but only return published
            all_posts = [
                {"title": "Published Post", "draft": False},
                {"title": "Draft Post", "draft": True},
                {"title": "Another Published", "draft": False},
            ]

            # Filter drafts
            return [cls(**p) for p in all_posts if not p["draft"]]

    posts = Post.load()
    assert len(posts) == 2
    assert all(not p.draft for p in posts)


# ==========================================
# Part 5: Pydantic Integration
# ==========================================

@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_custom_loader_with_pydantic():
    """Custom loader works with Pydantic models."""

    class User(Collection, pydantic.BaseModel):
        name: str
        email: str
        age: int

        @classmethod
        def load(cls) -> list['User']:
            data = [
                {"name": "Alice", "email": "alice@example.com", "age": 30},
                {"name": "Bob", "email": "bob@example.com", "age": 25},
            ]
            return [cls(**item) for item in data]

    users = User.load()
    assert len(users) == 2
    assert users[0].name == "Alice"
    assert users[0].email == "alice@example.com"
    assert users[0].age == 30


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_custom_loader_pydantic_compute_fields():
    """Pydantic + custom loader with computed fields."""

    class User(Collection, pydantic.BaseModel):
        name: str
        age: int
        name_upper: str = ""

        @classmethod
        def load(cls) -> list['User']:
            # Compute fields manually
            return [
                cls(name="alice", age=30, name_upper="ALICE"),
                cls(name="bob", age=25, name_upper="BOB"),
            ]

    users = User.load()
    assert users[0].name_upper == "ALICE"
    assert users[1].name_upper == "BOB"


# ==========================================
# Part 6: Edge Cases
# ==========================================

def test_custom_loader_empty_list():
    """Custom loader can return empty list."""

    class Post(Collection):
        title: str

        @classmethod
        def load(cls) -> list['Post']:
            return []

    posts = Post.load()
    assert posts == []


def test_custom_loader_single_item():
    """Custom loader can return single-item list."""

    class Post(Collection):
        title: str

        @classmethod
        def load(cls) -> list['Post']:
            return [cls(title="Only One")]

    posts = Post.load()
    assert len(posts) == 1
    assert posts[0].title == "Only One"


def test_custom_loader_no_meta_pattern_required():
    """Custom loaders don't need Meta.pattern."""

    class Post(Collection):
        title: str

        # No Meta.pattern!

        @classmethod
        def load(cls) -> list['Post']:
            return [cls(title="No pattern needed")]

    posts = Post.load()
    assert len(posts) == 1
    assert posts[0].title == "No pattern needed"
