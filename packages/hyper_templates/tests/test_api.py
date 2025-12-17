"""Tests for the public API surface and contracts."""

import pytest
from pathlib import Path

from hyper import render, load_component, slot
from hyper_templates.component import Component
from hyper_templates.errors import TemplateError, PropValidationError, ComponentNotFoundError
from hyper_templates._tdom.nodes import Node


class TestPublicAPI:
    """Tests that document and verify the public API."""

    def test_public_exports(self):
        """Public API exports render, load_component, and slot."""
        import hyper_templates as templates

        assert hasattr(templates, "render")
        assert hasattr(templates, "load_component")
        assert hasattr(templates, "slot")

        assert callable(templates.render)
        assert callable(templates.load_component)
        assert templates.slot is ...

    def test_slot_is_ellipsis(self):
        """slot is an alias for Python's Ellipsis (...)."""
        assert slot is ...
        assert slot is Ellipsis

    def test_render_signature(self):
        """render() accepts any value and returns str."""
        # String input
        assert isinstance(render("test"), str)

        # Number input
        assert isinstance(render(123), str)

        # None input
        assert isinstance(render(None), str)

    def test_load_component_signature(self, tmp_path: Path):
        """load_component() accepts Path or str, returns Component."""
        (tmp_path / "Test.py").write_text('t"""<div>Test</div>"""')

        # Path object
        comp1 = load_component(tmp_path / "Test.py")
        assert isinstance(comp1, Component)

        # String path
        comp2 = load_component(str(tmp_path / "Test.py"))
        assert isinstance(comp2, Component)

    def test_component_is_callable(self, tmp_path: Path):
        """Component instances are callable and return Node."""
        (tmp_path / "Test.py").write_text('t"""<div>Test</div>"""')

        component = load_component(tmp_path / "Test.py")

        assert callable(component)

        result = component()
        assert isinstance(result, Node)
        assert str(result) == "<div>Test</div>"

    def test_component_call_signature(self, tmp_path: Path):
        """Component() accepts children tuple and **kwargs."""
        (tmp_path / "Test.py").write_text("""
title: str = "Default"

t\"\"\"<div><h1>{title}</h1>{...}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")

        # No arguments
        component()

        # With children
        component(children=("<p>Content</p>",))

        # With props
        component(title="Custom")

        # With both
        component(title="Custom", children=("<p>Content</p>",))


class TestErrorHierarchy:
    """Tests for error class hierarchy and behavior."""

    def test_all_errors_inherit_from_template_error(self):
        """All custom errors inherit from TemplateError."""
        assert issubclass(PropValidationError, TemplateError)
        assert issubclass(ComponentNotFoundError, TemplateError)

    def test_template_error_is_exception(self):
        """TemplateError inherits from Exception."""
        assert issubclass(TemplateError, Exception)

    def test_prop_validation_error_fields(self, tmp_path: Path):
        """PropValidationError provides path, component_name, props."""
        (tmp_path / "Test.py").write_text("""
name: str

t\"\"\"<div>{name}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")

        try:
            component()  # Missing required prop
        except PropValidationError as e:
            assert e.path is not None
            assert e.component_name == "Test"
            assert isinstance(e.props, dict)
        else:
            pytest.fail("Expected PropValidationError")

    def test_component_not_found_error_fields(self):
        """ComponentNotFoundError provides path."""
        try:
            load_component(Path("/nonexistent/Component.py"))
        except ComponentNotFoundError as e:
            assert e.path is not None
            assert e.path == Path("/nonexistent/Component.py")
        else:
            pytest.fail("Expected ComponentNotFoundError")


class TestComponentAttributes:
    """Tests for Component object attributes."""

    def test_component_has_name(self, tmp_path: Path):
        """Component.name is derived from filename."""
        (tmp_path / "MyButton.py").write_text('t"""<button>Click</button>"""')

        component = load_component(tmp_path / "MyButton.py")

        assert component.name == "MyButton"

    def test_component_has_path(self, tmp_path: Path):
        """Component.path is the file path."""
        path = tmp_path / "Test.py"
        path.write_text('t"""<div>Test</div>"""')

        component = load_component(path)

        assert component.path == path

    def test_component_has_props(self, tmp_path: Path):
        """Component.props is a dict of Prop."""
        (tmp_path / "Test.py").write_text("""
title: str = "Hello"
count: int = 0

t\"\"\"<div>{title} - {count}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")

        assert isinstance(component.props, dict)
        assert "title" in component.props
        assert "count" in component.props

        # Prop has expected attributes
        title_prop = component.props["title"]
        assert hasattr(title_prop, "name")
        assert hasattr(title_prop, "type_hint")
        assert hasattr(title_prop, "default")
        assert hasattr(title_prop, "has_default")

        assert title_prop.name == "title"
        assert title_prop.type_hint == str
        assert title_prop.default == "Hello"
        assert title_prop.has_default is True

    def test_component_has_source(self, tmp_path: Path):
        """Component.source contains the original file content."""
        source_code = '''title: str = "Test"\nt"""<div>{title}</div>"""'''
        (tmp_path / "Test.py").write_text(source_code)

        component = load_component(tmp_path / "Test.py")

        assert component.code == source_code


class TestTypeCoercion:
    """Tests for how different types are handled."""

    def test_string_props_pass_through(self, tmp_path: Path):
        """String props are passed through unchanged."""
        (tmp_path / "Test.py").write_text("""
text: str = ""

t\"\"\"<div>{text}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")
        result = component(text="Hello World")

        assert "Hello World" in result

    def test_int_props_rendered_as_strings(self, tmp_path: Path):
        """Int props are rendered as strings."""
        (tmp_path / "Test.py").write_text("""
count: int = 0

t\"\"\"<div>{count}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")
        result = component(count=42)

        assert "42" in result

    def test_bool_props_rendered_as_strings(self, tmp_path: Path):
        """Bool props are converted to 'True' or 'False'."""
        (tmp_path / "Test.py").write_text("""
enabled: bool = False

t\"\"\"<div>{enabled}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")

        result_true = component(enabled=True)
        assert "True" in result_true

        result_false = component(enabled=False)
        assert "False" in result_false

    def test_none_props_render_as_empty(self, tmp_path: Path):
        """None props render as empty (tdom behavior)."""
        (tmp_path / "Test.py").write_text("""
value: str = "default"

t\"\"\"<div class="test">{value}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")
        result = component(value=None)

        # None renders as empty
        assert '<div class="test"></div>' in str(result)


class TestChildrenHandling:
    """Tests for how children are handled."""

    def test_children_must_be_tuple(self, tmp_path: Path):
        """Children should be provided as a tuple."""
        (tmp_path / "Test.py").write_text('t"""<div>{...}</div>"""')

        component = load_component(tmp_path / "Test.py")

        # Tuple works
        result = component(children=("<p>Test</p>",))
        assert "<p>Test</p>" in result

    def test_children_are_not_escaped(self, tmp_path: Path):
        """Children HTML is trusted and not escaped."""
        (tmp_path / "Test.py").write_text('t"""<div>{...}</div>"""')

        component = load_component(tmp_path / "Test.py")

        result = component(children=("<strong>Bold</strong>",))

        # Should not be escaped
        assert "<strong>Bold</strong>" in result
        assert "&lt;" not in result

    def test_multiple_children_concatenated(self, tmp_path: Path):
        """Multiple children are concatenated in order."""
        (tmp_path / "Test.py").write_text('t"""<div>{...}</div>"""')

        component = load_component(tmp_path / "Test.py")

        result = component(children=(
            "<p>First</p>",
            "<p>Second</p>",
            "<p>Third</p>",
        ))

        # Should appear in order
        assert result.index("First") < result.index("Second")
        assert result.index("Second") < result.index("Third")

    def test_empty_children_renders_empty_slot(self, tmp_path: Path):
        """Empty children tuple renders empty slot."""
        (tmp_path / "Test.py").write_text('t"""<div class="wrapper">{...}</div>"""')

        component = load_component(tmp_path / "Test.py")

        result = component(children=())

        assert '<div class="wrapper"></div>' in str(result)


class TestSecurityAndEscaping:
    """Tests for XSS prevention and HTML escaping."""

    def test_props_are_escaped(self, tmp_path: Path):
        """Props containing HTML are escaped."""
        (tmp_path / "Test.py").write_text("""
content: str = ""

t\"\"\"<div>{content}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")

        result = component(content='<script>alert("xss")</script>')

        # Should be escaped
        result_str = str(result)
        assert "<script>" not in result_str
        assert "&lt;script&gt;" in result_str or "&#60;script&#62;" in result_str

    def test_children_are_trusted_not_escaped(self, tmp_path: Path):
        """Children are trusted HTML (not escaped)."""
        (tmp_path / "Test.py").write_text('t"""<div>{...}</div>"""')

        component = load_component(tmp_path / "Test.py")

        result = component(children=("<script>console.log('ok')</script>",))

        # Children are NOT escaped - they are trusted
        assert "<script>" in result

    def test_special_chars_in_props_escaped(self, tmp_path: Path):
        """Special HTML characters in props are escaped."""
        (tmp_path / "Test.py").write_text("""
text: str = ""

t\"\"\"<div>{text}</div>\"\"\"
""")

        component = load_component(tmp_path / "Test.py")

        result = component(text='< > & " \'')

        # All special chars should be escaped
        result_str = str(result)
        assert "&lt;" in result_str
        assert "&gt;" in result_str
        assert "&amp;" in result_str
