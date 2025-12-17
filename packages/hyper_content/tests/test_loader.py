"""Library-agnostic test suite for hyper.content loader.

This file tests core functionality using only stdlib dataclasses.
Library-specific tests (msgspec, pydantic) are in separate files.
"""

from dataclasses import dataclass

import pytest

from hyper import Collection, Singleton, load


@pytest.fixture
def content_dir(tmp_path, monkeypatch):
    """Create and populate test content directory."""
    monkeypatch.chdir(tmp_path)

    (tmp_path / "settings.json").write_text('{"theme": "dark", "version": 1}')
    (tmp_path / "numbers.json").write_text('[1, 2, 3, 4, 5]')
    (tmp_path / "names.yaml").write_text('- Alice\n- Bob\n- Charlie')
    (tmp_path / "sponsors.json").write_text(
        '[{"name": "Google", "tier": "Gold"}, {"name": "GitHub", "tier": "Silver"}]'
    )

    config_dir = tmp_path / "config"
    config_dir.mkdir()
    (config_dir / "db.json").write_text('{"db_host": "localhost"}')
    (config_dir / "auth.yaml").write_text('api_key: secret-123')
    (config_dir / "cache.toml").write_text('cache_enabled = true')
    (config_dir / "timeout.json").write_text('{"timeout": 30}')

    blog_dir = tmp_path / "blog"
    blog_dir.mkdir()
    (blog_dir / "post-1.md").write_text("---\ntitle: Hello World\n---\nWelcome to my first post.")
    (blog_dir / "post-2.md").write_text("---\ntitle: Second Post\n---\nAnother update here.")

    return tmp_path


# ==========================================
# Part 1: Loading raw data (no type hints)
# ==========================================
def test_load_without_type_hint(content_dir):
    """Load files without type hints returns raw dicts/lists."""
    settings = load("settings.json")
    numbers = load("numbers.json")
    sponsors = load("sponsors.json")

    assert settings == {"theme": "dark", "version": 1}
    assert numbers == [1, 2, 3, 4, 5]
    assert sponsors == [{"name": "Google", "tier": "Gold"}, {"name": "GitHub", "tier": "Silver"}]


def test_load_multiple_files_without_type_hint(content_dir):
    """Loading multiple files without type hint returns list of dicts."""
    configs = load("config/*")

    assert isinstance(configs, list)
    assert len(configs) == 4


def test_load_list_of_primitives(content_dir):
    """Load lists of primitive types."""
    numbers = load("numbers.json", list[int])
    names = load("names.yaml", list[str])

    assert numbers == [1, 2, 3, 4, 5]
    assert names == ["Alice", "Bob", "Charlie"]


# ==========================================
# Part 2: Dataclasses (stdlib)
# ==========================================
def test_load_dataclass(content_dir):
    """Dataclasses work with type-safe loading."""
    @dataclass
    class Settings:
        theme: str
        version: int

    @dataclass
    class Sponsor:
        name: str
        tier: str

    settings = load("settings.json", Settings)
    sponsors = load("sponsors.json", list[Sponsor])

    assert settings.theme == "dark"
    assert len(sponsors) == 2


def test_bracket_syntax(content_dir):
    """Alternative syntax: load[Type](path) instead of load(path, Type)."""
    @dataclass
    class Settings:
        theme: str
        version: int

    settings = load[Settings]("settings.json")
    numbers = load[list[int]]("numbers.json")

    assert settings.version == 1
    assert numbers == [1, 2, 3, 4, 5]


# ==========================================
# Part 3: Multi-file merging
# ==========================================
def test_merge_files_into_singleton(content_dir):
    """Load and merge multiple files into a single object."""
    @dataclass
    class Config:
        db_host: str | None = None
        api_key: str | None = None
        cache_enabled: bool | None = None
        timeout: int | None = None

    config = load("config/*", Config)

    assert config.db_host == "localhost"
    assert config.api_key == "secret-123"
    assert config.cache_enabled is True
    assert config.timeout == 30


def test_merge_across_formats(content_dir):
    """Merging works across JSON, YAML, and TOML files."""
    @dataclass
    class Config:
        db_host: str | None = None
        api_key: str | None = None
        cache_enabled: bool | None = None

    config = load("config/*", Config)

    assert config.db_host == "localhost"
    assert config.api_key == "secret-123"
    assert config.cache_enabled is True


# ==========================================
# Part 4: Markdown with frontmatter
# ==========================================
def test_load_markdown_frontmatter(content_dir):
    """Markdown files with YAML frontmatter are parsed with .content and .html properties."""
    @dataclass
    class Post:
        id: str
        title: str
        body: str  # Changed from 'content' to 'body'
        html: str

    posts = load("blog/*.md", list[Post])

    assert len(posts) == 2
    assert posts[0].title == "Hello World"
    assert posts[0].body == "Welcome to my first post."  # Changed
    assert posts[0].html == "<p>Welcome to my first post.</p>"
    assert {p.id for p in posts} == {"blog/post-1", "blog/post-2"}  # Now includes path


# ==========================================
# Part 5: Singleton and Collection classes
# ==========================================
def test_singleton_and_collection_with_load(content_dir):
    """Singleton and Collection work with dataclasses."""
    @dataclass
    class Settings(Singleton):
        theme: str
        version: int

    @dataclass
    class Sponsor(Collection):
        name: str
        tier: str

    settings = load("settings.json", Settings)
    sponsors = load("sponsors.json", list[Sponsor])

    assert settings.theme == "dark"
    assert len(sponsors) == 2


# ==========================================
# Part 6: Class-based pattern with Meta
# ==========================================
def test_meta_pattern_singleton(content_dir):
    """Singleton with Meta.pattern can call .load() directly."""
    @dataclass
    class Settings(Singleton):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    settings = Settings.load()

    assert settings.theme == "dark"
    assert settings.version == 1


def test_meta_pattern_collection(content_dir):
    """Collection with Meta.pattern can call .load() directly."""
    @dataclass
    class Sponsor(Collection):
        name: str
        tier: str

        class Meta:
            pattern = "sponsors.json"

    sponsors = Sponsor.load()

    assert len(sponsors) == 2
    assert sponsors[0].name == "Google"


def test_meta_pattern_merge(content_dir):
    """Meta pattern works with multi-file merging."""
    @dataclass
    class Config(Singleton):
        db_host: str | None = None
        api_key: str | None = None
        cache_enabled: bool | None = None
        timeout: int | None = None

        class Meta:
            pattern = "config/*"

    config = Config.load()

    assert config.db_host == "localhost"
    assert config.cache_enabled is True


def test_meta_pattern_markdown(content_dir):
    """Meta pattern works with markdown collections."""
    @dataclass
    class Post(Collection):
        id: str
        title: str
        body: str  # Changed from 'content' to 'body'
        html: str

        class Meta:
            pattern = "blog/*.md"

    posts = Post.load()

    assert len(posts) == 2
    assert {p.id for p in posts} == {"blog/post-1", "blog/post-2"}  # Now includes path
    assert all(p.html for p in posts)


# ==========================================
# Part 7: Error handling
# ==========================================
def test_missing_file_error(content_dir):
    """Missing specific files raise FileNotFoundError."""
    @dataclass
    class Settings:
        theme: str

    with pytest.raises(FileNotFoundError):
        load("nonexistent.json", Settings)


def test_empty_glob_returns_empty_list(content_dir):
    """Non-matching glob patterns return empty list for collections."""
    @dataclass
    class Sponsor:
        name: str
        tier: str

    result = load("nonexistent/*.json", list[Sponsor])

    assert result == []


def test_list_file_into_singleton_fails(content_dir):
    """Loading a list file into a singleton type fails clearly."""
    @dataclass
    class Settings:
        theme: str

    with pytest.raises(ValueError, match="contains a list"):
        load("sponsors.json", Settings)


def test_load_no_matches_no_type_hint(tmp_path, monkeypatch):
    """Loading with glob that matches nothing and no type hint returns empty list."""
    monkeypatch.chdir(tmp_path)

    result = load("*.notfound")

    assert result == []


def test_unknown_file_extension(tmp_path, monkeypatch):
    """Unknown file extensions raise helpful error."""
    monkeypatch.chdir(tmp_path)
    (tmp_path / "data.xyz").write_text("some random content")

    with pytest.raises(ValueError, match=r"Unsupported file extension.*\.xyz.*Supported formats"):
        load("data.xyz")


def test_markdown_without_frontmatter_delimiters(tmp_path, monkeypatch):
    """Markdown without --- delimiters is treated as plain body with HTML."""
    monkeypatch.chdir(tmp_path)
    (tmp_path / "plain.md").write_text("# Hello\n\nJust some markdown content")

    data = load("plain.md")

    assert data["body"] == "# Hello\n\nJust some markdown content"  # Changed from 'content'
    assert "<h1>Hello</h1>" in data["html"]
    assert "<p>Just some markdown content</p>" in data["html"]


def test_markdown_with_malformed_frontmatter(tmp_path, monkeypatch):
    """Markdown with unclosed frontmatter falls back to body-only with HTML."""
    monkeypatch.chdir(tmp_path)
    (tmp_path / "bad.md").write_text("---\ntitle: Test\nno closing delimiter")

    data = load("bad.md")

    assert "body" in data  # Changed from 'content'
    assert "html" in data
    assert "---" in data["body"]  # Changed from 'content'


def test_metadata_injection_for_list_items(tmp_path, monkeypatch):
    """List items get _source metadata injected."""
    monkeypatch.chdir(tmp_path)
    (tmp_path / "items.json").write_text('[{"name": "A"}, {"name": "B"}]')

    @dataclass
    class Item:
        name: str
        _source: str | None = None

    items = load("items.json", list[Item])

    assert all(item._source == "items.json" for item in items)


def test_metadata_injection_skips_existing_id(tmp_path, monkeypatch):
    """If data already has 'id', don't override it."""
    monkeypatch.chdir(tmp_path)
    (tmp_path / "custom.json").write_text('{"id": "my-custom-id", "value": 42}')

    @dataclass
    class Data:
        id: str
        value: int

    data = load("custom.json", Data)

    assert data.id == "my-custom-id"


def test_no_data_found_singleton_error(tmp_path, monkeypatch):
    """Empty glob pattern for singleton raises ValueError."""
    monkeypatch.chdir(tmp_path)

    @dataclass
    class Settings:
        theme: str

    with pytest.raises(ValueError, match="No data found"):
        load("nonexistent/*.json", Settings)


def test_missing_meta_pattern_error():
    """Calling .load() without Meta.pattern raises helpful error."""
    @dataclass
    class NoMeta(Singleton):
        theme: str

    with pytest.raises(ValueError, match="Missing Meta.pattern or Meta.url on NoMeta"):
        NoMeta.load()

    @dataclass
    class NoMetaCollection(Collection):
        title: str

    with pytest.raises(ValueError, match="Missing Meta.pattern or Meta.url on NoMetaCollection"):
        NoMetaCollection.load()


# ==========================================
# Part 7: Msgspec Integration Tests
# ==========================================

try:
    import msgspec
    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_singleton_with_explicit_msgspec_struct(simple_json_singleton):
    """Singleton works with explicit msgspec.Struct inheritance."""

    class Settings(Singleton, msgspec.Struct):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    assert issubclass(Settings, msgspec.Struct)
    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.version == 1


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_collection_with_explicit_msgspec_struct(simple_json_collection):
    """Collection works with explicit msgspec.Struct inheritance."""

    class Post(Collection, msgspec.Struct):
        title: str
        views: int

        class Meta:
            pattern = "posts/*.json"

    assert issubclass(Post, msgspec.Struct)
    posts = Post.load()
    assert len(posts) == 2
    assert all(isinstance(p, Post) for p in posts)


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_msgspec_validation_during_load(content_dir):
    """msgspec validates during decode/load."""
    (content_dir / "invalid.json").write_text('{"theme": 123, "version": "not-int"}')

    class InvalidSettings(Singleton, msgspec.Struct):
        theme: str
        version: int

        class Meta:
            pattern = "invalid.json"

    with pytest.raises(msgspec.ValidationError):
        InvalidSettings.load()


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_singleton_without_msgspec_becomes_dataclass(simple_json_singleton):
    """When msgspec.Struct is not used, Singleton becomes auto-dataclass."""
    from dataclasses import is_dataclass

    class Settings(Singleton):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    assert is_dataclass(Settings)
    assert not issubclass(Settings, msgspec.Struct)

    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.version == 1


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_collection_without_msgspec_becomes_dataclass(simple_json_collection):
    """When msgspec.Struct is not used, Collection becomes auto-dataclass."""
    from dataclasses import is_dataclass

    class Post(Collection):
        title: str
        views: int

        class Meta:
            pattern = "posts/*.json"

    assert is_dataclass(Post)
    assert not issubclass(Post, msgspec.Struct)

    posts = Post.load()
    assert len(posts) == 2


# ==========================================
# Part 8: Pydantic Integration Tests
# ==========================================

try:
    import pydantic
    from pydantic import BaseModel, ValidationError, field_validator
    HAS_PYDANTIC = True
except ImportError:
    HAS_PYDANTIC = False
    BaseModel = object
    ValidationError = Exception


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_singleton_with_explicit_pydantic_basemodel(simple_json_singleton):
    """Singleton works with explicit pydantic.BaseModel inheritance."""

    class Settings(Singleton, BaseModel):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    assert issubclass(Settings, BaseModel)
    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.version == 1


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_collection_with_explicit_pydantic_basemodel(simple_json_collection):
    """Collection works with explicit pydantic.BaseModel inheritance."""

    class Post(Collection, BaseModel):
        title: str
        views: int

        class Meta:
            pattern = "posts/*.json"

    assert issubclass(Post, BaseModel)
    posts = Post.load()
    assert len(posts) == 2
    assert all(isinstance(p, Post) for p in posts)


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_pydantic_validation_during_load(content_dir):
    """pydantic validates during load."""
    (content_dir / "invalid.json").write_text('{"theme": 123, "version": "not-int"}')

    class InvalidSettings(Singleton, BaseModel):
        theme: str
        version: int

        class Meta:
            pattern = "invalid.json"

    with pytest.raises(ValidationError):
        InvalidSettings.load()


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_pydantic_field_validators_work(simple_json_singleton):
    """Pydantic-specific features like field validators work with explicit BaseModel."""

    class Settings(Singleton, BaseModel):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

        @field_validator("theme")
        @classmethod
        def validate_theme(cls, v: str) -> str:
            if v not in ["dark", "light"]:
                raise ValueError("theme must be 'dark' or 'light'")
            return v

    settings = Settings.load()
    assert settings.theme == "dark"


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_pydantic_field_validators_fail_on_invalid(content_dir):
    """Pydantic field validators reject invalid data."""
    (content_dir / "bad.json").write_text('{"theme": "invalid", "version": 1}')

    class BadSettings(Singleton, BaseModel):
        theme: str
        version: int

        class Meta:
            pattern = "bad.json"

        @field_validator("theme")
        @classmethod
        def validate_theme(cls, v: str) -> str:
            if v not in ["dark", "light"]:
                raise ValueError("theme must be 'dark' or 'light'")
            return v

    with pytest.raises(ValidationError, match="theme must be"):
        BadSettings.load()


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_singleton_without_pydantic_becomes_dataclass(simple_json_singleton):
    """When pydantic.BaseModel is not used, Singleton becomes auto-dataclass."""
    from dataclasses import is_dataclass

    class Settings(Singleton):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    assert is_dataclass(Settings)
    assert not issubclass(Settings, BaseModel)

    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.version == 1


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_collection_without_pydantic_becomes_dataclass(simple_json_collection):
    """When pydantic.BaseModel is not used, Collection becomes auto-dataclass."""
    from dataclasses import is_dataclass

    class Post(Collection):
        title: str
        views: int

        class Meta:
            pattern = "posts/*.json"

    assert is_dataclass(Post)
    assert not issubclass(Post, BaseModel)

    posts = Post.load()
    assert len(posts) ==2


# ==========================================
# Part 9: Edge Cases and Real-World Scenarios
# ==========================================

from typing import Optional
from dataclasses import field

try:
    from pydantic import ConfigDict, Field
except ImportError:
    ConfigDict = lambda **k: None
    Field = lambda **k: None

try:
    import msgspec
    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False

from hyper import Singleton, Collection


# ===========================================
# Optional Fields with Defaults
# ===========================================


def test_optional_fields_with_defaults_dataclass(optional_fields_json):
    """Dataclass with optional fields and defaults works correctly."""

    class Config(Singleton):
        name: str
        debug: bool = False
        port: int = 3000
        host: str = "localhost"

        class Meta:
            pattern = "settings/minimal.json"

    config = Config.load()
    assert config.name == "Development"
    assert config.debug is False  # default value
    assert config.port == 3000  # default value
    assert config.host == "localhost"  # default value


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_optional_fields_with_defaults_pydantic(optional_fields_json):
    """Pydantic with optional fields and defaults works correctly."""

    class Config(Singleton, BaseModel):
        name: str
        debug: bool = False
        port: int = 3000
        host: str = "localhost"

        class Meta:
            pattern = "settings/minimal.json"

    config = Config.load()
    assert config.name == "Development"
    assert config.debug is False
    assert config.port == 3000
    assert config.host == "localhost"


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_optional_fields_with_defaults_msgspec(optional_fields_json):
    """Msgspec with optional fields and defaults works correctly."""

    class Config(Singleton, msgspec.Struct):
        name: str
        debug: bool = False
        port: int = 3000
        host: str = "localhost"

        class Meta:
            pattern = "settings/minimal.json"

    config = Config.load()
    assert config.name == "Development"
    assert config.debug is False
    assert config.port == 3000
    assert config.host == "localhost"


# ===========================================
# Extra/Unknown Fields Handling
# ===========================================


def test_extra_fields_ignored_dataclass(extra_fields_json):
    """Dataclass ignores extra fields that aren't defined."""

    class Data(Singleton):
        known_field: str
        another_known: int

        class Meta:
            pattern = "data.json"

    data = Data.load()
    assert data.known_field == "value"
    assert data.another_known == 123
    # Extra fields are simply not present


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_extra_fields_ignored_pydantic(extra_fields_json):
    """Pydantic with extra='ignore' ignores unknown fields."""

    class Data(Singleton, BaseModel):
        model_config = ConfigDict(extra='ignore')

        known_field: str
        another_known: int

        class Meta:
            pattern = "data.json"

    data = Data.load()
    assert data.known_field == "value"
    assert data.another_known == 123


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_extra_fields_ignored_msgspec(extra_fields_json):
    """Msgspec ignores extra fields by default."""

    class Data(Singleton, msgspec.Struct):
        known_field: str
        another_known: int

        class Meta:
            pattern = "data.json"

    data = Data.load()
    assert data.known_field == "value"
    assert data.another_known == 123


# ===========================================
# Empty Collections
# ===========================================


def test_empty_collection_returns_empty_list(empty_collection):
    """Loading from empty directory returns empty list."""

    class Post(Collection):
        title: str

        class Meta:
            pattern = "empty/*.json"

    posts = Post.load()
    assert posts == []
    assert isinstance(posts, list)


# ===========================================
# Mixed File Formats
# ===========================================


def test_mixed_formats_in_collection(content_dir):
    """Collection can load from JSON and YAML simultaneously."""
    configs_dir = content_dir / "configs"
    configs_dir.mkdir()

    (configs_dir / "app.json").write_text('{"name": "app", "type": "json"}')
    (configs_dir / "db.yaml").write_text('name: db\ntype: yaml\n')

    class Config(Collection):
        name: str
        type: str

        class Meta:
            pattern = "configs/*.*"

    configs = Config.load()
    assert len(configs) == 2

    names = {c.name for c in configs}
    assert names == {"app", "db"}

    types = {c.type for c in configs}
    assert types == {"json", "yaml"}


# ===========================================
# Explicit @dataclass Decorator
# ===========================================


def test_explicit_dataclass_decorator(simple_json_singleton):
    """User can explicitly use @dataclass decorator."""

    @dataclass
    class Settings(Singleton):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    # Should already be a dataclass (no double-wrapping)
    from dataclasses import is_dataclass
    assert is_dataclass(Settings)

    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.version == 1


def test_dataclass_with_field_factory(content_dir):
    """Dataclass with field(default_factory=...) works correctly."""

    (content_dir / "list.json").write_text('{"items": [1, 2, 3]}')

    @dataclass
    class Data(Singleton):
        items: list[int] = field(default_factory=list)
        tags: list[str] = field(default_factory=list)

        class Meta:
            pattern = "list.json"

    data = Data.load()
    assert data.items == [1, 2, 3]
    assert data.tags == []  # default factory


# ===========================================
# Multiple Inheritance Patterns
# ===========================================


def test_custom_base_class_with_singleton(simple_json_singleton):
    """Custom base class can be combined with Singleton."""

    class HasTimestamp:
        """Mixin that adds timestamp functionality."""

        def get_timestamp(self):
            return "2024-01-01"

    class Settings(HasTimestamp, Singleton):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.get_timestamp() == "2024-01-01"


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_mixin_pydantic_singleton(simple_json_singleton):
    """Mixin + Singleton + Pydantic inheritance works."""

    class HasMethods:
        def uppercase_theme(self):
            return self.theme.upper()

    class Settings(HasMethods, Singleton, BaseModel):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    settings = Settings.load()
    assert settings.theme == "dark"
    assert settings.uppercase_theme() == "DARK"


# ===========================================
# Type Coercion Edge Cases
# ===========================================


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_pydantic_type_coercion(type_coercion_json):
    """Pydantic automatically coerces types."""

    class Numbers(Singleton, BaseModel):
        string_number: int  # "123" → 123
        float_as_int: int  # 42.0 → 42
        bool_as_string: bool  # "true" → True
        null_value: Optional[str]  # null → None

        class Meta:
            pattern = "numbers.json"

    nums = Numbers.load()
    assert nums.string_number == 123
    assert nums.float_as_int == 42
    assert nums.bool_as_string is True
    assert nums.null_value is None


# ===========================================
# Nested Models
# ===========================================


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_deeply_nested_models(nested_json):
    """Deeply nested pydantic models work correctly."""

    class Credentials(BaseModel):
        user: str
        password: str

    class Database(BaseModel):
        host: str
        port: int
        credentials: Credentials

    class Cache(BaseModel):
        enabled: bool
        ttl: int

    class Config(Singleton, BaseModel):
        database: Database
        cache: Cache

        class Meta:
            pattern = "config.json"

    config = Config.load()
    assert config.database.host == "localhost"
    assert config.database.port == 5432
    assert config.database.credentials.user == "admin"
    assert config.database.credentials.password == "secret"
    assert config.cache.enabled is True
    assert config.cache.ttl == 300


# ===========================================
# Union Types
# ===========================================


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_union_types_pydantic(content_dir):
    """Pydantic handles Union types correctly."""
    from typing import Union

    (content_dir / "flexible.json").write_text('{"value": 42}')

    class Flexible(Singleton, BaseModel):
        value: Union[str, int, float]

        class Meta:
            pattern = "flexible.json"

    data = Flexible.load()
    assert data.value == 42
    assert isinstance(data.value, int)


# ===========================================
# File Not Found vs Empty Pattern
# ===========================================


def test_nonexistent_singleton_file_raises_error(content_dir):
    """Loading non-existent singleton file raises FileNotFoundError."""

    class Missing(Singleton):
        data: str

        class Meta:
            pattern = "does-not-exist.json"

    with pytest.raises(FileNotFoundError, match="does-not-exist.json"):
        Missing.load()


def test_nonexistent_collection_pattern_returns_empty_list(content_dir):
    """Loading non-existent collection glob pattern returns empty list."""

    class Missing(Collection):
        data: str

        class Meta:
            pattern = "nonexistent-dir/*.json"

    # Glob patterns with no matches return empty list
    result = Missing.load()
    assert result == []


def test_empty_glob_pattern_vs_nonexistent(content_dir):
    """Empty glob (no matches) returns empty list, but non-glob raises error."""

    (content_dir / "empty").mkdir()

    # Glob pattern with no matches → empty list
    class EmptyPattern(Collection):
        title: str

        class Meta:
            pattern = "empty/*.json"

    result = EmptyPattern.load()
    assert result == []

    # Non-glob pattern that doesn't exist → error
    class NonExistent(Singleton):
        title: str

        class Meta:
            pattern = "missing.json"

    with pytest.raises(FileNotFoundError):
        NonExistent.load()


# ===========================================
# Multiple Classes Using Same Pattern
# ===========================================


def test_multiple_classes_same_pattern(simple_json_singleton):
    """Multiple class definitions can use the same pattern independently."""

    class Settings1(Singleton):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    class Settings2(Singleton):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    s1 = Settings1.load()
    s2 = Settings2.load()

    # Both should load successfully
    assert s1.theme == s2.theme == "dark"
    assert s1.version == s2.version == 1

    # But they're different classes
    assert type(s1) is not type(s2)
    assert type(s1).__name__ == "Settings1"
    assert type(s2).__name__ == "Settings2"


# ===========================================
# Validation with Different Libraries
# ===========================================


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_pydantic_custom_validators(validation_test_json):
    """Pydantic custom validators work correctly."""
    from pydantic import field_validator

    class User(Singleton, BaseModel):
        email: str
        age: int

        class Meta:
            pattern = "data/valid.json"

        @field_validator("email")
        @classmethod
        def validate_email(cls, v: str) -> str:
            if "@" not in v:
                raise ValueError("Invalid email")
            return v

        @field_validator("age")
        @classmethod
        def validate_age(cls, v: int) -> int:
            if v < 0:
                raise ValueError("Age must be positive")
            return v

    # Valid data works
    user = User.load()
    assert user.email == "user@example.com"
    assert user.age == 25

    # Invalid email
    class InvalidEmail(Singleton, BaseModel):
        email: str
        age: int

        class Meta:
            pattern = "data/invalid_email.json"

        @field_validator("email")
        @classmethod
        def validate_email(cls, v: str) -> str:
            if "@" not in v:
                raise ValueError("Invalid email")
            return v

    with pytest.raises(pydantic.ValidationError, match="Invalid email"):
        InvalidEmail.load()


# ===========================================
# Collection Ordering
# ===========================================


def test_collection_ordering_is_stable(content_dir):
    """Collection items are returned in consistent order (sorted by path)."""

    posts_dir = content_dir / "posts"
    posts_dir.mkdir()

    # Create files in a specific order
    (posts_dir / "c-third.json").write_text('{"title": "Third", "order": 3}')
    (posts_dir / "a-first.json").write_text('{"title": "First", "order": 1}')
    (posts_dir / "b-second.json").write_text('{"title": "Second", "order": 2}')

    class Post(Collection):
        title: str
        order: int

        class Meta:
            pattern = "posts/*.json"

    posts = Post.load()

    # Should be ordered alphabetically by filename
    assert len(posts) == 3
    assert posts[0].title == "First"
    assert posts[1].title == "Second"
    assert posts[2].title == "Third"


# ===========================================
# Immutable Models (Frozen)
# ===========================================


@pytest.mark.skipif(not HAS_PYDANTIC, reason="pydantic not installed")
def test_frozen_pydantic_model(simple_json_singleton):
    """Frozen pydantic models are immutable after creation."""

    class Settings(Singleton, BaseModel):
        model_config = ConfigDict(frozen=True)

        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    settings = Settings.load()
    assert settings.theme == "dark"

    # Should not be able to modify
    with pytest.raises((pydantic.ValidationError, AttributeError)):
        settings.theme = "light"


@pytest.mark.skipif(not HAS_MSGSPEC, reason="msgspec not installed")
def test_frozen_msgspec_struct(simple_json_singleton):
    """Frozen msgspec structs are immutable after creation."""

    class Settings(Singleton, msgspec.Struct, frozen=True):
        theme: str
        version: int

        class Meta:
            pattern = "settings.json"

    settings = Settings.load()
    assert settings.theme == "dark"

    # Should not be able to modify
    with pytest.raises(AttributeError):
        settings.theme = "light"
