"""Tests for complex real-world scenarios."""

import os
from typing import Any

import pytest
import pydantic
from pydantic import BaseModel, field_validator

from hyper.content import Collection, Singleton, load

try:
    import msgspec

    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False


@pytest.fixture
def content_dir(tmp_path):
    """Set up test content directory with sample files."""
    content = tmp_path / "content"
    content.mkdir()
    os.chdir(content)
    yield content


# ===========================================
# Nested Models Tests
# ===========================================


def test_nested_pydantic_models(content_dir):
    """Pydantic automatically validates nested models."""

    class Author(BaseModel):
        name: str
        email: str

    class BlogPost(Collection, pydantic.BaseModel):
        title: str
        author: Author  # Nested pydantic model
        tags: list[str]

        class Meta:
            pattern = "posts/*.json"

    # Create test data
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text(
        '{"title": "Hello", "author": {"name": "Alice", "email": "alice@example.com"}, "tags": ["python", "tutorial"]}'
    )

    posts = BlogPost.load()
    assert len(posts) == 1
    assert isinstance(posts[0].author, Author)
    assert posts[0].author.name == "Alice"
    assert posts[0].author.email == "alice@example.com"
    assert posts[0].tags == ["python", "tutorial"]


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_nested_msgspec_models(content_dir):
    """Msgspec automatically validates nested structs."""

    class Author(msgspec.Struct):
        name: str
        email: str

    class BlogPost(Collection, msgspec.Struct):
        title: str
        author: Author  # Nested msgspec struct
        tags: list[str]

        class Meta:
            pattern = "posts/*.json"

    # Create test data
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text(
        '{"title": "Hello", "author": {"name": "Alice", "email": "alice@example.com"}, "tags": ["python", "tutorial"]}'
    )

    posts = BlogPost.load()
    assert len(posts) == 1
    assert isinstance(posts[0].author, Author)
    assert posts[0].author.name == "Alice"
    assert posts[0].tags == ["python", "tutorial"]


# ===========================================
# Validation Tests
# ===========================================


def test_pydantic_field_validators(content_dir):
    """Pydantic field validators run automatically."""

    class User(Singleton, pydantic.BaseModel):
        email: str
        age: int

        @field_validator("email")
        @classmethod
        def validate_email(cls, v):
            if "@" not in v:
                raise ValueError("Invalid email")
            return v.lower()

        @field_validator("age")
        @classmethod
        def validate_age(cls, v):
            if v < 0 or v > 150:
                raise ValueError("Invalid age")
            return v

        class Meta:
            pattern = "user.json"

    # Valid data
    (content_dir / "user.json").write_text('{"email": "Alice@Example.com", "age": 30}')
    user = User.load()
    assert user.email == "alice@example.com"  # Lowercased by validator
    assert user.age == 30

    # Invalid email
    (content_dir / "invalid-email.json").write_text(
        '{"email": "notanemail", "age": 30}'
    )
    with pytest.raises(ValueError, match="Invalid email"):
        load("invalid-email.json", User)

    # Invalid age
    (content_dir / "invalid-age.json").write_text(
        '{"email": "test@example.com", "age": 200}'
    )
    with pytest.raises(ValueError, match="Invalid age"):
        load("invalid-age.json", User)


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_post_init_validation(content_dir):
    """Msgspec __post_init__ can be used for custom validation."""

    class Config(Singleton, msgspec.Struct):
        api_key: str
        rate_limit: int

        def __post_init__(self):
            if len(self.api_key) < 10:
                raise ValueError("API key too short")
            if self.rate_limit <= 0:
                raise ValueError("Rate limit must be positive")

        class Meta:
            pattern = "config.json"

    # Valid data
    (content_dir / "config.json").write_text(
        '{"api_key": "secret1234567", "rate_limit": 100}'
    )
    config = Config.load()
    assert config.api_key == "secret1234567"
    assert config.rate_limit == 100

    # Invalid api_key (too short) - msgspec wraps the error
    (content_dir / "invalid-key.json").write_text(
        '{"api_key": "short", "rate_limit": 100}'
    )
    with pytest.raises((ValueError, msgspec.ValidationError)):
        load("invalid-key.json", Config)

    # Invalid rate_limit - msgspec wraps the error
    (content_dir / "invalid-rate.json").write_text(
        '{"api_key": "secret1234567", "rate_limit": -1}'
    )
    with pytest.raises((ValueError, msgspec.ValidationError)):
        load("invalid-rate.json", Config)


# ===========================================
# Computed Properties Tests
# ===========================================


def test_computed_properties(content_dir):
    """Properties can compute derived values."""

    class Product(Collection, pydantic.BaseModel):
        name: str
        price: float
        tax_rate: float = 0.1

        @property
        def total_price(self) -> float:
            return self.price * (1 + self.tax_rate)

        class Meta:
            pattern = "products/*.json"

    # Create test data
    products_dir = content_dir / "products"
    products_dir.mkdir()
    (products_dir / "item1.json").write_text(
        '{"name": "Laptop", "price": 1000.0, "tax_rate": 0.15}'
    )
    (products_dir / "item2.json").write_text('{"name": "Mouse", "price": 50.0}')

    products = Product.load()
    assert len(products) == 2

    # First product with custom tax rate
    laptop = next(p for p in products if p.name == "Laptop")
    assert laptop.total_price == pytest.approx(1150.0)  # 1000 * 1.15

    # Second product with default tax rate
    mouse = next(p for p in products if p.name == "Mouse")
    assert mouse.total_price == pytest.approx(55.0)  # 50 * 1.1


# ===========================================
# Default Values & Optional Fields Tests
# ===========================================


def test_default_values_and_optional_fields(content_dir):
    """Models can have default values and optional fields."""

    class Settings(Singleton, pydantic.BaseModel):
        theme: str = "light"
        language: str = "en"
        notifications: bool = True
        custom_css: str | None = None

        class Meta:
            pattern = "settings.json"

    # Partial data - should use defaults
    (content_dir / "settings.json").write_text('{"theme": "dark"}')
    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.language == "en"  # default
    assert settings.notifications is True  # default
    assert settings.custom_css is None  # default

    # All fields provided
    (content_dir / "full-settings.json").write_text(
        '{"theme": "blue", "language": "fr", "notifications": false, "custom_css": "body { color: red; }"}'
    )
    settings2 = load("full-settings.json", Settings)
    assert settings2.theme == "blue"
    assert settings2.language == "fr"
    assert settings2.notifications is False
    assert settings2.custom_css == "body { color: red; }"


# ===========================================
# Complex Types Tests
# ===========================================


def test_complex_types(content_dir):
    """Models can have lists, dicts, and nested structures."""

    class Project(Singleton, pydantic.BaseModel):
        name: str
        tags: list[str]
        metadata: dict[str, Any]
        dependencies: list[dict[str, str]]

        class Meta:
            pattern = "project.json"

    # Create test data with complex nested structures
    (content_dir / "project.json").write_text(
        """{
            "name": "my-project",
            "tags": ["python", "web", "api"],
            "metadata": {
                "version": "1.0.0",
                "author": "Alice",
                "config": {
                    "debug": true,
                    "port": 8000
                }
            },
            "dependencies": [
                {"name": "fastapi", "version": "^0.100.0"},
                {"name": "pydantic", "version": "^2.0.0"}
            ]
        }"""
    )

    project = Project.load()
    assert project.name == "my-project"
    assert project.tags == ["python", "web", "api"]
    assert project.metadata["version"] == "1.0.0"
    assert project.metadata["config"]["debug"] is True
    assert len(project.dependencies) == 2
    assert project.dependencies[0]["name"] == "fastapi"


# ===========================================
# Merge Strategy Tests
# ===========================================


def test_merge_strategy_shallow(content_dir):
    """Shallow merge overwrites top-level keys."""

    class Config(Singleton, pydantic.BaseModel):
        app: dict[str, Any]
        features: dict[str, bool]

        class Meta:
            pattern = "config/*.json"

    # Create multiple config files (name them so base loads first alphabetically)
    config_dir = content_dir / "config"
    config_dir.mkdir()
    (config_dir / "00_base.json").write_text(
        '{"app": {"name": "MyApp", "version": "1.0"}, "features": {"auth": true, "api": true}}'
    )
    (config_dir / "99_override.json").write_text(
        '{"app": {"version": "2.0"}, "features": {"api": false, "admin": true}}'
    )

    # Shallow merge - second file overwrites entire nested dicts
    config = Config.load()  # default is shallow
    assert config.app == {"version": "2.0"}  # "name" is lost with shallow merge!
    assert config.features == {"api": False, "admin": True}  # "auth" is lost!


def test_merge_strategy_deep(content_dir):
    """Deep merge recursively merges nested dictionaries."""

    class Config(Singleton, pydantic.BaseModel):
        app: dict[str, Any]
        features: dict[str, bool]

        class Meta:
            pattern = "config/*.json"

    # Create multiple config files (name them so base loads first alphabetically)
    config_dir = content_dir / "config"
    config_dir.mkdir()
    (config_dir / "00_base.json").write_text(
        '{"app": {"name": "MyApp", "version": "1.0"}, "features": {"auth": true, "api": true}}'
    )
    (config_dir / "99_override.json").write_text(
        '{"app": {"version": "2.0"}, "features": {"api": false, "admin": true}}'
    )

    # Deep merge - recursively merges nested dicts
    config = load("config/*.json", Config, merge="deep")
    assert config.app == {"name": "MyApp", "version": "2.0"}  # Both keys present!
    assert config.features == {
        "auth": True,
        "api": False,
        "admin": True,
    }  # All keys present!
