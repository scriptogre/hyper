"""Shared test fixtures for all test modules."""

import os
from pathlib import Path
import pytest


@pytest.fixture
def content_dir(tmp_path):
    """Create and navigate to a temporary content directory.

    This is the base fixture used by most tests. It creates a clean
    temporary directory and changes the working directory to it.
    """
    content = tmp_path / "content"
    content.mkdir()
    os.chdir(content)
    yield content


@pytest.fixture
def simple_json_singleton(content_dir):
    """Create a simple JSON file for Singleton testing."""
    (content_dir / "settings.json").write_text(
        '{"theme": "dark", "version": 1}'
    )
    return content_dir


@pytest.fixture
def simple_json_collection(content_dir):
    """Create simple JSON files for Collection testing."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()
    (posts_dir / "hello.json").write_text('{"title": "Hello", "views": 100}')
    (posts_dir / "world.json").write_text('{"title": "World", "views": 200}')
    return content_dir


@pytest.fixture
def nested_json(content_dir):
    """Create JSON with nested objects."""
    (content_dir / "config.json").write_text('''
    {
        "database": {
            "host": "localhost",
            "port": 5432,
            "credentials": {
                "user": "admin",
                "password": "secret"
            }
        },
        "cache": {
            "enabled": true,
            "ttl": 300
        }
    }
    ''')
    return content_dir


@pytest.fixture
def optional_fields_json(content_dir):
    """Create JSON with some missing optional fields."""
    settings_dir = content_dir / "settings"
    settings_dir.mkdir()

    # Full config
    (settings_dir / "full.json").write_text(
        '{"name": "Production", "debug": false, "port": 8080, "host": "0.0.0.0"}'
    )

    # Minimal config (only required fields)
    (settings_dir / "minimal.json").write_text(
        '{"name": "Development"}'
    )

    return content_dir


@pytest.fixture
def extra_fields_json(content_dir):
    """Create JSON with extra unknown fields."""
    (content_dir / "data.json").write_text('''
    {
        "known_field": "value",
        "another_known": 123,
        "unknown_extra": "should be ignored",
        "nested_unknown": {"key": "value"}
    }
    ''')
    return content_dir


@pytest.fixture
def mixed_formats(content_dir):
    """Create content in multiple formats (JSON, YAML, TOML)."""
    configs_dir = content_dir / "configs"
    configs_dir.mkdir()

    (configs_dir / "app.json").write_text('{"name": "app", "type": "json"}')
    (configs_dir / "db.yaml").write_text('name: db\ntype: yaml\n')
    (configs_dir / "cache.toml").write_text('[cache]\nname = "cache"\ntype = "toml"\n')

    return content_dir


@pytest.fixture
def empty_collection(content_dir):
    """Create an empty directory for testing empty collections."""
    empty_dir = content_dir / "empty"
    empty_dir.mkdir()
    return content_dir


@pytest.fixture
def markdown_files(content_dir):
    """Create markdown files with frontmatter."""
    posts_dir = content_dir / "posts"
    posts_dir.mkdir()

    (posts_dir / "hello.md").write_text('''---
title: Hello World
slug: hello-world
published: true
---

# Hello World

This is the content.
''')

    (posts_dir / "draft.md").write_text('''---
title: Draft Post
slug: draft-post
published: false
---

# Draft

This is a draft.
''')

    return content_dir


@pytest.fixture
def type_coercion_json(content_dir):
    """Create JSON with types that need coercion."""
    (content_dir / "numbers.json").write_text('''
    {
        "string_number": "123",
        "float_as_int": 42.0,
        "bool_as_string": "true",
        "null_value": null
    }
    ''')
    return content_dir


@pytest.fixture
def invalid_json(content_dir):
    """Create an invalid JSON file."""
    (content_dir / "broken.json").write_text('{invalid json content')
    return content_dir


@pytest.fixture
def validation_test_json(content_dir):
    """Create JSON files for testing validation (valid and invalid)."""
    data_dir = content_dir / "data"
    data_dir.mkdir()

    # Valid data
    (data_dir / "valid.json").write_text('{"email": "user@example.com", "age": 25}')

    # Invalid email
    (data_dir / "invalid_email.json").write_text('{"email": "not-an-email", "age": 25}')

    # Invalid age (negative)
    (data_dir / "invalid_age.json").write_text('{"email": "user@example.com", "age": -5}')

    return content_dir
