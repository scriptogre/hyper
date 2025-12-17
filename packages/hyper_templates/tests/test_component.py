"""Tests for component loading and invocation."""

import pytest
from pathlib import Path

from hyper_templates.component import Component, load_component
from hyper_templates.errors import ComponentNotFoundError, PropValidationError
from hyper_templates._tdom.nodes import Node


class TestLoadComponent:
    """Tests for load_component function."""

    def test_load_simple_component(self, tmp_path: Path):
        """Load a .py file as a component."""
        (tmp_path / "Button.py").write_text('t"""<button>{...}</button>"""')

        component = load_component(tmp_path / "Button.py")

        assert component.name == "Button"
        assert component.path == tmp_path / "Button.py"

    def test_load_component_extracts_props(self, tmp_path: Path):
        """Component extracts props from type hints."""
        (tmp_path / "Layout.py").write_text('''
title: str = "My Site"

t"""
<html>
<head><title>{title}</title></head>
<body>{...}</body>
</html>
"""
''')

        component = load_component(tmp_path / "Layout.py")

        assert "title" in component.props
        assert component.props["title"].default == "My Site"
        assert component.props["title"].type_hint == str

    def test_load_component_not_found(self):
        """ComponentNotFoundError for missing file."""
        with pytest.raises(ComponentNotFoundError) as exc:
            load_component(Path("/nonexistent/Component.py"))

        assert "Component.py" in str(exc.value)

    def test_load_component_multiple_props(self, tmp_path: Path):
        """Component with multiple props."""
        (tmp_path / "Card.py").write_text('''
title: str = "Card Title"
subtitle: str = ""
active: bool = False
count: int = 0

t"""<div class="card">{title}</div>"""
''')

        component = load_component(tmp_path / "Card.py")

        assert len(component.props) == 4
        assert component.props["title"].default == "Card Title"
        assert component.props["subtitle"].default == ""
        assert component.props["active"].default is False
        assert component.props["count"].default == 0


class TestComponentInvocation:
    """Tests for calling components."""

    def test_component_with_default_props(self, tmp_path: Path):
        """Calling component uses default prop values."""
        (tmp_path / "Greeting.py").write_text('''
name: str = "World"

t"""<h1>Hello, {name}!</h1>"""
''')

        component = load_component(tmp_path / "Greeting.py")
        # Should not raise - uses default
        result = component()

        assert isinstance(result, Node)

    def test_component_with_provided_props(self, tmp_path: Path):
        """Calling component with prop values."""
        (tmp_path / "Greeting.py").write_text('''
name: str = "World"

t"""<h1>Hello, {name}!</h1>"""
''')

        component = load_component(tmp_path / "Greeting.py")
        result = component(name="Alice")

        assert isinstance(result, Node)

    def test_component_missing_required_prop(self, tmp_path: Path):
        """PropValidationError for missing required prop."""
        (tmp_path / "Required.py").write_text('''
name: str

t"""<h1>{name}</h1>"""
''')

        component = load_component(tmp_path / "Required.py")

        with pytest.raises(PropValidationError) as exc:
            component()

        assert "name" in str(exc.value)
        assert "Required.py" in str(exc.value)

    def test_component_wrong_prop_type(self, tmp_path: Path):
        """PropValidationError for wrong prop type."""
        (tmp_path / "Typed.py").write_text('''
count: int = 0

t"""<span>{count}</span>"""
''')

        component = load_component(tmp_path / "Typed.py")

        with pytest.raises(PropValidationError) as exc:
            component(count="not an int")

        assert "int" in str(exc.value)


class TestComponentWithChildren:
    """Tests for components with slot content."""

    def test_component_accepts_children(self, tmp_path: Path):
        """Component can receive children tuple."""
        (tmp_path / "Wrapper.py").write_text('''
t"""<div class="wrapper">{...}</div>"""
''')

        component = load_component(tmp_path / "Wrapper.py")
        # Should not raise
        result = component(children=("Content",))

        assert isinstance(result, Node)
