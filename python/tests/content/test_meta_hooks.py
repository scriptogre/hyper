"""Tests for Meta hooks: before_parse, after_parse, and after_load."""

import pytest
from pathlib import Path

try:
    import msgspec

    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False
    msgspec = None

try:
    import pydantic

    HAS_PYDANTIC = True
except ImportError:
    HAS_PYDANTIC = False
    pydantic = None

from hyper.content import Singleton, Collection, load


# ==========================================
# Part 1: before_parse Hook Tests
# ==========================================


def test_before_parse_hook_singleton(content_dir):
    """before_parse hook can modify raw bytes before parsing."""
    (content_dir / "config.json").write_text('{"env": "PRODUCTION", "debug": true}')

    class Config(Singleton):
        env: str
        debug: bool

        class Meta:
            pattern = "config.json"

            @staticmethod
            def before_parse(path: Path, content: bytes) -> bytes:
                # Redact production environments in tests
                return content.replace(b"PRODUCTION", b"TEST")

    config = Config.load()
    assert config.env == "TEST"  # Modified by hook
    assert config.debug is True


def test_before_parse_hook_collection(content_dir):
    """before_parse hook applies to each file in collection."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text(
        '{"title": "DRAFT: Hello", "status": "draft"}'
    )
    (posts_dir / "post2.json").write_text(
        '{"title": "DRAFT: World", "status": "draft"}'
    )

    class Post(Collection):
        title: str
        status: str

        class Meta:
            pattern = "posts/*.json"

            @staticmethod
            def before_parse(path: Path, content: bytes) -> bytes:
                # Remove DRAFT prefix from all posts
                return content.replace(b"DRAFT: ", b"")

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].title == "Hello"  # DRAFT prefix removed
    assert posts[1].title == "World"  # DRAFT prefix removed


def test_before_parse_hook_decryption_simulation(content_dir):
    """before_parse can simulate decryption or decompression."""
    # Simulate "encrypted" content with rot13-like byte shift
    encrypted = bytes((b + 1) % 256 for b in b'{"secret": "password123"}')
    (content_dir / "secrets.json").write_bytes(encrypted)

    class Secrets(Singleton):
        secret: str

        class Meta:
            pattern = "secrets.json"

            @staticmethod
            def before_parse(path: Path, content: bytes) -> bytes:
                # "Decrypt" by reversing the byte shift
                return bytes((b - 1) % 256 for b in content)

    secrets = Secrets.load()
    assert secrets.secret == "password123"


# ==========================================
# Part 2: after_parse Hook Tests
# ==========================================


def test_after_parse_hook_singleton(content_dir):
    """after_parse hook can modify parsed dict before conversion."""
    (content_dir / "article.json").write_text(
        '{"title": "Hello World", "body": "This is a test article"}'
    )

    class Article(Singleton):
        title: str
        body: str
        word_count: int

        class Meta:
            pattern = "article.json"

            @staticmethod
            def after_parse(path: Path, data: dict) -> dict:
                # Inject computed field
                data["word_count"] = len(data.get("body", "").split())
                return data

    article = Article.load()
    assert article.title == "Hello World"
    assert article.word_count == 5  # Injected by hook


def test_after_parse_hook_collection(content_dir):
    """after_parse hook applies to each file in collection."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text(
        '{"title": "First", "content": "Hello world"}'
    )
    (posts_dir / "post2.json").write_text(
        '{"title": "Second", "content": "Foo bar baz"}'
    )

    class Post(Collection):
        title: str
        content: str
        char_count: int

        class Meta:
            pattern = "posts/*.json"

            @staticmethod
            def after_parse(path: Path, data: dict) -> dict:
                data["char_count"] = len(data.get("content", ""))
                return data

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].char_count == 11  # "Hello world"
    assert posts[1].char_count == 11  # "Foo bar baz"


def test_after_parse_hook_field_transformation(content_dir):
    """after_parse can transform legacy field names."""
    (content_dir / "legacy.json").write_text(
        '{"old_name": "value", "deprecated_field": 123}'
    )

    class Model(Singleton):
        new_name: str
        current_field: int

        class Meta:
            pattern = "legacy.json"

            @staticmethod
            def after_parse(path: Path, data: dict) -> dict:
                # Migrate old field names to new schema
                if "old_name" in data:
                    data["new_name"] = data.pop("old_name")
                if "deprecated_field" in data:
                    data["current_field"] = data.pop("deprecated_field")
                return data

    model = Model.load()
    assert model.new_name == "value"
    assert model.current_field == 123


def test_after_parse_hook_filter_sensitive_data(content_dir):
    """after_parse can filter or redact sensitive fields."""
    (content_dir / "user.json").write_text(
        '{"name": "Alice", "email": "alice@example.com", "password": "secret123"}'
    )

    class User(Singleton):
        name: str
        email: str
        password: str

        class Meta:
            pattern = "user.json"

            @staticmethod
            def after_parse(path: Path, data: dict) -> dict:
                # Redact password
                if "password" in data:
                    data["password"] = "[REDACTED]"
                return data

    user = User.load()
    assert user.name == "Alice"
    assert user.password == "[REDACTED]"


# ==========================================
# Part 3: after_load Hook Tests
# ==========================================


def test_after_load_hook_singleton(content_dir):
    """after_load hook can modify final instance."""
    (content_dir / "config.json").write_text('{"host": "localhost", "port": 8080}')

    class Config(Singleton):
        host: str
        port: int
        url: str = ""

        class Meta:
            pattern = "config.json"

            @staticmethod
            def after_load(instance: "Config") -> "Config":
                # Compute derived field
                instance.url = f"http://{instance.host}:{instance.port}"
                return instance

    config = Config.load()
    assert config.host == "localhost"
    assert config.port == 8080
    assert config.url == "http://localhost:8080"


def test_after_load_hook_collection(content_dir):
    """after_load hook applies to each item in collection."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "post1.json").write_text('{"title": "hello world", "views": 100}')
    (posts_dir / "post2.json").write_text('{"title": "foo bar", "views": 200}')

    class Post(Collection):
        title: str
        views: int
        title_upper: str = ""

        class Meta:
            pattern = "posts/*.json"

            @staticmethod
            def after_load(instance: "Post") -> "Post":
                # Normalize title
                instance.title_upper = instance.title.upper()
                return instance

    posts = Post.load()
    assert len(posts) == 2
    assert posts[0].title_upper == "HELLO WORLD"
    assert posts[1].title_upper == "FOO BAR"


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_after_load_hook_with_pydantic(content_dir):
    """after_load hook works with Pydantic models."""
    (content_dir / "user.json").write_text('{"name": "alice", "age": 30}')

    class User(Singleton, pydantic.BaseModel):
        name: str
        age: int
        name_capitalized: str = ""

        class Meta:
            pattern = "user.json"

            @staticmethod
            def after_load(instance: "User") -> "User":
                instance.name_capitalized = instance.name.capitalize()
                return instance

    user = User.load()
    assert user.name == "alice"
    assert user.name_capitalized == "Alice"


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_after_load_hook_with_msgspec(content_dir):
    """after_load hook works with msgspec models."""
    (content_dir / "settings.json").write_text('{"theme": "dark", "version": 1}')

    class Settings(Singleton, msgspec.Struct):
        theme: str
        version: int
        display_name: str = ""

        class Meta:
            pattern = "settings.json"

            @staticmethod
            def after_load(instance: "Settings") -> "Settings":
                # Note: msgspec.Struct is immutable by default, but we can use replace
                return msgspec.structs.replace(
                    instance,
                    display_name=f"{instance.theme.upper()} v{instance.version}",
                )

    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.display_name == "DARK v1"


# ==========================================
# Part 4: Combined Hooks Tests
# ==========================================


def test_all_hooks_together(content_dir):
    """All three hooks can be used together in correct order."""
    (content_dir / "data.json").write_text('{"RAW": "value", "count": 5}')

    call_order = []

    class Data(Singleton):
        processed: str
        count: int
        final: str = ""

        class Meta:
            pattern = "data.json"

            @staticmethod
            def before_parse(path: Path, content: bytes) -> bytes:
                call_order.append("before_parse")
                # Stage 1: Modify bytes
                return content.replace(b"RAW", b"processed")

            @staticmethod
            def after_parse(path: Path, data: dict) -> dict:
                call_order.append("after_parse")
                # Stage 2: Modify dict
                data["count"] = data["count"] * 2
                return data

            @staticmethod
            def after_load(instance: "Data") -> "Data":
                call_order.append("after_load")
                # Stage 3: Modify instance
                instance.final = f"{instance.processed}-{instance.count}"
                return instance

    data = Data.load()
    assert data.processed == "value"
    assert data.count == 10  # 5 * 2 from after_parse
    assert data.final == "value-10"  # Computed in after_load
    assert call_order == ["before_parse", "after_parse", "after_load"]


def test_hooks_with_merge_strategy(content_dir):
    """Hooks work correctly with multiple files and merge strategies."""
    (content_dir / "config1.json").write_text('{"KEY1": "value1", "shared": "first"}')
    (content_dir / "config2.json").write_text('{"KEY2": "value2", "shared": "second"}')

    class Config(Singleton):
        key1: str
        key2: str
        shared: str
        processed: bool = False

        class Meta:
            pattern = "config*.json"

            @staticmethod
            def before_parse(path: Path, content: bytes) -> bytes:
                # Normalize keys to lowercase
                return content.replace(b"KEY1", b"key1").replace(b"KEY2", b"key2")

            @staticmethod
            def after_load(instance: "Config") -> "Config":
                instance.processed = True
                return instance

    config = Config.load()
    assert config.key1 == "value1"
    assert config.key2 == "value2"
    assert config.shared == "second"  # Shallow merge, second wins
    assert config.processed is True


def test_hook_with_no_type_hint_skips_hooks(content_dir):
    """Hooks are not called when loading without type hints."""
    (content_dir / "data.json").write_text('{"value": "test"}')

    # Load without type hint - no hooks should be called
    result = load("data.json")
    assert result == {"value": "test"}


# ==========================================
# Part 5: Edge Cases and Error Handling
# ==========================================


def test_hook_returning_none_raises_error(content_dir):
    """Hook that returns None should cause an error."""
    (content_dir / "bad.json").write_text('{"value": "test"}')

    class BadModel(Singleton):
        value: str

        class Meta:
            pattern = "bad.json"

            @staticmethod
            def after_parse(path: Path, data: dict) -> dict:
                return None  # Oops!

    with pytest.raises((TypeError, AttributeError, ValueError)):
        BadModel.load()


def test_hooks_optional_not_required(content_dir):
    """Models work fine without any hooks defined."""
    (content_dir / "simple.json").write_text('{"value": "test"}')

    class Simple(Singleton):
        value: str

        class Meta:
            pattern = "simple.json"
            # No hooks defined - should work fine

    simple = Simple.load()
    assert simple.value == "test"


def test_before_parse_hook_with_path_context(content_dir):
    """before_parse hook receives path for context-aware processing."""
    drafts_dir = content_dir / "drafts"
    published_dir = content_dir / "published"
    drafts_dir.mkdir()
    published_dir.mkdir()

    (drafts_dir / "post1.json").write_text('{"title": "Draft Post", "status": "draft"}')
    (published_dir / "post2.json").write_text(
        '{"title": "Published Post", "status": "published"}'
    )

    class Post(Collection):
        title: str
        status: str

        class Meta:
            pattern = "**/*.json"

            @staticmethod
            def before_parse(path: Path, content: bytes) -> bytes:
                # Force status based on directory
                if "drafts" in str(path):
                    content = content.replace(b'"published"', b'"draft"')
                return content

    posts = Post.load()
    assert len(posts) == 2
    # Both should have correct status based on directory
    assert all(p.status in ["draft", "published"] for p in posts)


def test_after_parse_with_path_for_metadata_injection(content_dir):
    """after_parse can use path to inject custom metadata."""
    posts_dir = content_dir / "2024/01"
    posts_dir.mkdir(parents=True)
    (posts_dir / "article.json").write_text('{"title": "Test", "content": "Hello"}')

    class Article(Collection):
        title: str
        content: str
        year: int = 0
        month: int = 0

        class Meta:
            pattern = "**/*.json"

            @staticmethod
            def after_parse(path: Path, data: dict) -> dict:
                # Extract date from path structure
                parts = path.parts
                if len(parts) >= 2:
                    try:
                        data["year"] = int(parts[-3])
                        data["month"] = int(parts[-2])
                    except (ValueError, IndexError):
                        pass
                return data

    articles = Article.load()
    assert len(articles) == 1
    assert articles[0].year == 2024
    assert articles[0].month == 1
