"""Tests for computed fields using @computed decorator."""

import pytest
from pathlib import Path

try:
    import pydantic
    HAS_PYDANTIC = True
except ImportError:
    HAS_PYDANTIC = False
    pydantic = None

try:
    import msgspec
    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False
    msgspec = None

from hyper.content import Singleton, Collection, computed


# ==========================================
# Part 1: Basic Computed Fields
# ==========================================

def test_computed_field_basic(content_dir):
    """Basic computed field is lazily evaluated."""
    (content_dir / "article.json").write_text('{"title": "Test", "body": "Hello world this is a test"}')

    import os
    os.chdir(content_dir)

    class Article(Singleton):
        title: str
        body: str

        class Meta:
            pattern = "article.json"

        @computed
        def word_count(self) -> int:
            return len(self.body.split())

    article = Article.load()
    assert article.title == "Test"
    assert article.body == "Hello world this is a test"
    assert article.word_count == 6


def test_computed_field_multiple(content_dir):
    """Multiple computed fields work together."""
    body_text = "This is a test article with some words"
    (content_dir / "post.json").write_text(f'{{"title": "Hello", "body": "{body_text}"}}')

    import os
    os.chdir(content_dir)

    class Post(Singleton):
        title: str
        body: str

        class Meta:
            pattern = "post.json"

        @computed
        def word_count(self) -> int:
            return len(self.body.split())

        @computed
        def char_count(self) -> int:
            return len(self.body)

        @computed
        def title_upper(self) -> str:
            return self.title.upper()

    post = Post.load()
    assert post.word_count == 8
    assert post.char_count == len(body_text)
    assert post.title_upper == "HELLO"


def test_computed_field_chaining(content_dir):
    """Computed fields can reference other computed fields."""
    # Create body with 400 words
    body = " ".join(["word"] * 400)
    import json
    (content_dir / "article.json").write_text(json.dumps({"title": "Test", "body": body}))

    import os
    os.chdir(content_dir)

    class Article(Singleton):
        title: str
        body: str

        class Meta:
            pattern = "article.json"

        @computed
        def word_count(self) -> int:
            return len(self.body.split())

        @computed
        def reading_time(self) -> str:
            # References another computed field
            minutes = self.word_count // 200
            return f"{minutes} min read"

    article = Article.load()
    assert article.word_count == 400
    assert article.reading_time == "2 min read"


# ==========================================
# Part 2: Computed Fields with Collections
# ==========================================

def test_computed_field_collection(content_dir):
    """Computed fields work with collections."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text('{"title": "First", "body": "Hello world"}')
    (posts_dir / "post2.json").write_text('{"title": "Second", "body": "Foo bar baz"}')

    import os
    os.chdir(content_dir)

    class Post(Collection):
        title: str
        body: str

        class Meta:
            pattern = "posts/*.json"

        @computed
        def word_count(self) -> int:
            return len(self.body.split())

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].word_count == 2  # "Hello world"
    assert posts[1].word_count == 3  # "Foo bar baz"


def test_computed_field_collection_complex(content_dir):
    """Computed fields with chaining in collections."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text('{"title": "article one", "views": 1000}')
    (posts_dir / "post2.json").write_text('{"title": "article two", "views": 500}')

    import os
    os.chdir(content_dir)

    class Post(Collection):
        title: str
        views: int

        class Meta:
            pattern = "posts/*.json"

        @computed
        def title_capitalized(self) -> str:
            return self.title.title()

        @computed
        def popularity(self) -> str:
            if self.views > 1000:
                return "viral"
            elif self.views > 500:
                return "popular"
            else:
                return "normal"

        @computed
        def display_name(self) -> str:
            return f"{self.title_capitalized} ({self.popularity})"

    posts = Post.load()
    assert posts[0].display_name == "Article One (popular)"
    assert posts[1].display_name == "Article Two (normal)"


# ==========================================
# Part 3: Caching Behavior
# ==========================================

def test_computed_field_caching(content_dir):
    """Computed fields are cached after first access."""
    (content_dir / "data.json").write_text('{"value": 10}')

    import os
    os.chdir(content_dir)

    call_count = {"count": 0}

    class Data(Singleton):
        value: int

        class Meta:
            pattern = "data.json"

        @computed
        def expensive_calculation(self) -> int:
            call_count["count"] += 1
            return self.value * 2

    data = Data.load()

    # First access
    result1 = data.expensive_calculation
    assert result1 == 20
    assert call_count["count"] == 1

    # Second access (should use cache)
    result2 = data.expensive_calculation
    assert result2 == 20
    assert call_count["count"] == 1  # Not incremented!

    # Third access (still cached)
    result3 = data.expensive_calculation
    assert result3 == 20
    assert call_count["count"] == 1


def test_computed_field_independent_caching(content_dir):
    """Each instance has its own cache."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text('{"title": "First", "value": 10}')
    (posts_dir / "post2.json").write_text('{"title": "Second", "value": 20}')

    import os
    os.chdir(content_dir)

    class Post(Collection):
        title: str
        value: int

        class Meta:
            pattern = "posts/*.json"

        @computed
        def doubled(self) -> int:
            return self.value * 2

    posts = Post.load()

    # Access first post
    assert posts[0].doubled == 20

    # Access second post
    assert posts[1].doubled == 40

    # Re-access first post (should still be cached correctly)
    assert posts[0].doubled == 20


# ==========================================
# Part 4: Integration with Validation Libraries
# ==========================================

@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_computed_field_with_pydantic(content_dir):
    """Computed fields work with Pydantic models."""
    (content_dir / "user.json").write_text('{"first_name": "John", "last_name": "Doe", "age": 30}')

    import os
    os.chdir(content_dir)

    class User(Singleton, pydantic.BaseModel):
        first_name: str
        last_name: str
        age: int

        class Meta:
            pattern = "user.json"

        @computed
        def full_name(self) -> str:
            return f"{self.first_name} {self.last_name}"

        @computed
        def is_adult(self) -> bool:
            return self.age >= 18

    user = User.load()
    assert user.full_name == "John Doe"
    assert user.is_adult is True


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_computed_field_with_msgspec(content_dir):
    """Computed fields work with msgspec.Struct."""
    (content_dir / "settings.json").write_text('{"theme": "dark", "font_size": 14}')

    import os
    os.chdir(content_dir)

    class Settings(Singleton, msgspec.Struct):
        theme: str
        font_size: int

        class Meta:
            pattern = "settings.json"

        @computed
        def theme_display(self) -> str:
            return f"{self.theme.upper()} Theme"

        @computed
        def is_large_font(self) -> bool:
            return self.font_size > 12

    settings = Settings.load()
    assert settings.theme_display == "DARK Theme"
    assert settings.is_large_font is True


# ==========================================
# Part 5: Custom Loaders + Computed Fields
# ==========================================

def test_computed_field_with_custom_loader():
    """Computed fields work with custom loaders."""

    class Post(Collection):
        title: str
        views: int

        @classmethod
        def load(cls) -> list['Post']:
            return [
                cls(title="First", views=100),
                cls(title="Second", views=200),
            ]

        @computed
        def views_formatted(self) -> str:
            if self.views >= 1000:
                return f"{self.views // 1000}k"
            return str(self.views)

        @computed
        def title_upper(self) -> str:
            return self.title.upper()

    posts = Post.load()
    assert posts[0].views_formatted == "100"
    assert posts[0].title_upper == "FIRST"
    assert posts[1].views_formatted == "200"
    assert posts[1].title_upper == "SECOND"


# ==========================================
# Part 6: Edge Cases
# ==========================================

def test_computed_field_with_none_values(content_dir):
    """Computed fields handle None values gracefully."""
    (content_dir / "data.json").write_text('{"name": "Test", "description": null}')

    import os
    os.chdir(content_dir)

    class Data(Singleton):
        name: str
        description: str | None

        class Meta:
            pattern = "data.json"

        @computed
        def description_length(self) -> int:
            return len(self.description) if self.description else 0

        @computed
        def has_description(self) -> bool:
            return self.description is not None and len(self.description) > 0

    data = Data.load()
    assert data.description_length == 0
    assert data.has_description is False


def test_computed_field_with_complex_types(content_dir):
    """Computed fields can work with lists, dicts, etc."""
    (content_dir / "data.json").write_text('{"tags": ["python", "web", "api"], "metadata": {"author": "John"}}')

    import os
    os.chdir(content_dir)

    class Data(Singleton):
        tags: list[str]
        metadata: dict[str, str]

        class Meta:
            pattern = "data.json"

        @computed
        def tag_count(self) -> int:
            return len(self.tags)

        @computed
        def tags_upper(self) -> list[str]:
            return [tag.upper() for tag in self.tags]

        @computed
        def author(self) -> str:
            return self.metadata.get("author", "Unknown")

    data = Data.load()
    assert data.tag_count == 3
    assert data.tags_upper == ["PYTHON", "WEB", "API"]
    assert data.author == "John"


def test_computed_field_error_propagation(content_dir):
    """Errors in computed fields are raised properly."""
    (content_dir / "data.json").write_text('{"value": 0}')

    import os
    os.chdir(content_dir)

    class Data(Singleton):
        value: int

        class Meta:
            pattern = "data.json"

        @computed
        def will_error(self) -> int:
            return 100 // self.value  # Division by zero!

    data = Data.load()

    with pytest.raises(ZeroDivisionError):
        _ = data.will_error
