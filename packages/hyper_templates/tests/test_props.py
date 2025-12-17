"""Tests for prop extraction from Python source."""

import pytest
from hyper_templates.loader import extract_props, Prop


class TestExtractProps:
    """Tests for extract_props function."""

    def test_extract_simple_prop(self):
        """Single prop with type and default."""
        source = 'title: str = "Hello"'
        props = extract_props(source)

        assert "title" in props
        assert props["title"].name == "title"
        assert props["title"].type_name == "str"
        assert props["title"].type_hint is None  # Not resolved yet
        assert props["title"].default == "Hello"
        assert props["title"].has_default is True

    def test_extract_prop_no_default(self):
        """Prop with type but no default is required."""
        source = "count: int"
        props = extract_props(source)

        assert "count" in props
        assert props["count"].name == "count"
        assert props["count"].type_name == "int"
        assert props["count"].type_hint is None  # Not resolved yet
        assert props["count"].has_default is False

    def test_extract_multiple_props(self):
        """Multiple props extracted correctly."""
        source = '''
title: str = "My Site"
count: int = 0
enabled: bool
'''
        props = extract_props(source)

        assert len(props) == 3
        assert props["title"].default == "My Site"
        assert props["count"].default == 0
        assert props["enabled"].has_default is False

    def test_ignore_non_annotated_assignments(self):
        """Regular assignments are not props."""
        source = '''
title: str = "Hello"
helper = "not a prop"
'''
        props = extract_props(source)

        assert "title" in props
        assert "helper" not in props

    def test_ignore_private_names(self):
        """Private/dunder names are not props."""
        source = '''
title: str = "Hello"
_private: str = "ignored"
__dunder__: str = "ignored"
'''
        props = extract_props(source)

        assert "title" in props
        assert "_private" not in props
        assert "__dunder__" not in props

    def test_extract_bool_prop(self):
        """Boolean prop with default."""
        source = "active: bool = True"
        props = extract_props(source)

        assert props["active"].type_name == "bool"
        assert props["active"].type_hint is None  # Not resolved yet
        assert props["active"].default is True

    def test_extract_list_default(self):
        """List default value."""
        source = 'tags: list = ["python", "web"]'
        props = extract_props(source)

        assert props["tags"].default == ["python", "web"]

    def test_extract_dict_default(self):
        """Dict default value."""
        source = 'config: dict = {"key": "value"}'
        props = extract_props(source)

        assert props["config"].default == {"key": "value"}

    def test_empty_source(self):
        """Empty source returns empty dict."""
        props = extract_props("")
        assert props == {}

    def test_invalid_syntax_returns_empty(self):
        """Invalid Python syntax returns empty dict."""
        source = "this is not valid python {"
        props = extract_props(source)
        assert props == {}

    def test_multiline_source(self):
        """Props extracted from multiline source with other code."""
        source = '''
# A comment
title: str = "Hello"

def some_function():
    pass

count: int = 42

class SomeClass:
    pass
'''
        props = extract_props(source)

        assert len(props) == 2
        assert props["title"].default == "Hello"
        assert props["count"].default == 42


class TestProp:
    """Tests for Prop dataclass."""

    def test_required_factory(self):
        """Prop.required creates required prop."""
        prop = Prop.required("name", str)

        assert prop.name == "name"
        assert prop.type_hint == str
        assert prop.has_default is False

    def test_with_default_factory(self):
        """Prop.with_default creates prop with default."""
        prop = Prop.with_default("title", "Hello", str)

        assert prop.name == "title"
        assert prop.type_hint == str
        assert prop.default == "Hello"
        assert prop.has_default is True
