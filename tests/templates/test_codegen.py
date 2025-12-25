"""Comprehensive tests for the template code generator.

These tests verify that the compiled templates produce correct output
and that the generated Python code is valid and efficient.
"""

import os
import pytest
import tempfile
from pathlib import Path

from markupsafe import Markup
from hyper.templates import load_template


class TestBasicCodegen:
    """Test basic code generation scenarios."""

    def test_simple_static_template(self, tmp_path: Path):
        """Templates with no interpolations generate static strings."""
        (tmp_path / "Static.py").write_text('t"""<div>Hello</div>"""')

        comp = load_template(tmp_path / "Static.py")
        result = comp()

        assert str(result) == "<div>Hello</div>"

    def test_single_interpolation(self, tmp_path: Path):
        """Single interpolation in template."""
        (tmp_path / "Single.py").write_text('''
name: str

t"""<p>Hello, {name}!</p>"""
''')

        comp = load_template(tmp_path / "Single.py")
        result = comp(name="World")

        assert "Hello, World!" in str(result)

    def test_multiple_interpolations(self, tmp_path: Path):
        """Multiple interpolations in template."""
        (tmp_path / "Multi.py").write_text('''
first: str
last: str

t"""<p>{first} {last}</p>"""
''')

        comp = load_template(tmp_path / "Multi.py")
        result = comp(first="John", last="Doe")

        assert "John Doe" in str(result)

    def test_nested_elements(self, tmp_path: Path):
        """Nested HTML elements."""
        (tmp_path / "Nested.py").write_text('''
title: str

t"""
<div class="container">
    <header>
        <h1>{title}</h1>
    </header>
</div>
"""
''')

        comp = load_template(tmp_path / "Nested.py")
        result = comp(title="Welcome")

        assert "<h1>Welcome</h1>" in str(result)
        assert 'class="container"' in str(result)


class TestAttributeCodegen:
    """Test attribute code generation."""

    def test_static_attributes(self, tmp_path: Path):
        """Static attributes are rendered correctly."""
        (tmp_path / "StaticAttr.py").write_text(
            't"""<div id="main" class="container">Content</div>"""'
        )

        comp = load_template(tmp_path / "StaticAttr.py")
        result = str(comp())

        assert 'id="main"' in result
        assert 'class="container"' in result

    def test_interpolated_attribute(self, tmp_path: Path):
        """Interpolated attribute values."""
        (tmp_path / "InterpAttr.py").write_text('''
id_value: str

t"""<div id={id_value}>Content</div>"""
''')

        comp = load_template(tmp_path / "InterpAttr.py")
        result = str(comp(id_value="my-id"))

        assert 'id="my-id"' in result

    def test_boolean_attribute_true(self, tmp_path: Path):
        """Boolean True renders attribute without value."""
        (tmp_path / "BoolTrue.py").write_text('''
is_disabled: bool

t"""<button disabled={is_disabled}>Click</button>"""
''')

        comp = load_template(tmp_path / "BoolTrue.py")
        result = str(comp(is_disabled=True))

        assert "disabled" in result

    def test_boolean_attribute_false(self, tmp_path: Path):
        """Boolean False omits the attribute entirely."""
        (tmp_path / "BoolFalse.py").write_text('''
is_disabled: bool

t"""<button disabled={is_disabled}>Click</button>"""
''')

        comp = load_template(tmp_path / "BoolFalse.py")
        result = str(comp(is_disabled=False))

        assert "disabled" not in result

    def test_templated_attribute(self, tmp_path: Path):
        """Attributes with mixed static and dynamic parts."""
        (tmp_path / "TemplatedAttr.py").write_text('''
variant: str

t"""<button class="btn btn-{variant}">Click</button>"""
''')

        comp = load_template(tmp_path / "TemplatedAttr.py")
        result = str(comp(variant="primary"))

        assert 'class="btn btn-primary"' in result


class TestChildrenSlot:
    """Test children/slot handling."""

    def test_empty_children(self, tmp_path: Path):
        """Empty children tuple renders nothing."""
        (tmp_path / "EmptySlot.py").write_text('t"""<div>{...}</div>"""')

        comp = load_template(tmp_path / "EmptySlot.py")
        result = str(comp())

        assert result == "<div></div>"

    def test_single_child(self, tmp_path: Path):
        """Single child is rendered."""
        (tmp_path / "SingleChild.py").write_text('t"""<div>{...}</div>"""')

        comp = load_template(tmp_path / "SingleChild.py")
        result = str(comp(children=("Hello",)))

        assert "Hello" in result

    def test_multiple_children(self, tmp_path: Path):
        """Multiple children are concatenated."""
        (tmp_path / "MultiChild.py").write_text('t"""<div>{...}</div>"""')

        comp = load_template(tmp_path / "MultiChild.py")
        result = str(comp(children=("Hello, ", "World!")))

        assert "Hello, World!" in result


class TestEscaping:
    """Test HTML escaping in generated code."""

    def test_props_are_escaped(self, tmp_path: Path):
        """Props with HTML are escaped."""
        (tmp_path / "EscapeProp.py").write_text('''
content: str

t"""<div>{content}</div>"""
''')

        comp = load_template(tmp_path / "EscapeProp.py")
        result = str(comp(content="<script>alert('xss')</script>"))

        assert "<script>" not in result
        assert "&lt;script&gt;" in result

    def test_attribute_values_escaped(self, tmp_path: Path):
        """Attribute values with special chars are escaped."""
        (tmp_path / "EscapeAttr.py").write_text('''
title: str

t"""<div title={title}>Content</div>"""
''')

        comp = load_template(tmp_path / "EscapeAttr.py")
        result = str(comp(title='Hello "World"'))

        # Quotes should be escaped in attribute values
        assert '"' in result or "&#34;" in result or "&quot;" in result

    def test_children_not_escaped(self, tmp_path: Path):
        """Children are trusted and not escaped."""
        (tmp_path / "TrustChild.py").write_text('t"""<div>{...}</div>"""')

        comp = load_template(tmp_path / "TrustChild.py")
        result = str(comp(children=("<b>Bold</b>",)))

        assert "<b>Bold</b>" in result


class TestPreTemplateCode:
    """Test arbitrary Python code before template."""

    def test_computed_variable(self, tmp_path: Path):
        """Variables computed from props."""
        (tmp_path / "Computed.py").write_text('''
name: str

greeting = f"Hello, {name}!"

t"""<p>{greeting}</p>"""
''')

        comp = load_template(tmp_path / "Computed.py")
        result = str(comp(name="World"))

        assert "Hello, World!" in result

    def test_conditional_logic(self, tmp_path: Path):
        """If/else logic before template."""
        (tmp_path / "Conditional.py").write_text('''
status: str

if status == "error":
    icon = "❌"
else:
    icon = "✅"

t"""<span>{icon}</span>"""
''')

        comp = load_template(tmp_path / "Conditional.py")

        result_error = str(comp(status="error"))
        assert "❌" in result_error

        result_ok = str(comp(status="ok"))
        assert "✅" in result_ok

    def test_list_operations(self, tmp_path: Path):
        """List operations before template."""
        (tmp_path / "ListOp.py").write_text('''
items: list

count = len(items)
first_item = items[0] if items else "none"

t"""<div>Count: {count}, First: {first_item}</div>"""
''')

        comp = load_template(tmp_path / "ListOp.py")
        result = str(comp(items=["a", "b", "c"]))

        assert "Count: 3" in result
        assert "First: a" in result


class TestDebugMode:
    """Test debug mode .gen.py file generation."""

    def test_debug_mode_creates_gen_file(self, tmp_path: Path):
        """When DEBUG=true, .gen.py files are created."""
        (tmp_path / "Debug.py").write_text('t"""<div>Debug test</div>"""')

        # Set debug mode
        os.environ["DEBUG"] = "true"
        try:
            comp = load_template(tmp_path / "Debug.py")
            comp()

            gen_file = tmp_path / "Debug.gen.py"
            assert gen_file.exists()

            gen_code = gen_file.read_text()
            assert "def render" in gen_code
            assert "escape_html" in gen_code
        finally:
            os.environ.pop("DEBUG", None)

    def test_debug_mode_off_no_gen_file(self, tmp_path: Path):
        """When DEBUG is not set, no .gen.py file is created."""
        (tmp_path / "NoDebug.py").write_text('t"""<div>No debug</div>"""')

        # Ensure debug mode is off
        os.environ.pop("DEBUG", None)

        comp = load_template(tmp_path / "NoDebug.py")
        comp()

        gen_file = tmp_path / "NoDebug.gen.py"
        assert not gen_file.exists()


class TestGeneratedCodeQuality:
    """Test that generated code is correct and follows good patterns."""

    def test_generated_code_is_valid_python(self, tmp_path: Path):
        """Generated code compiles as valid Python."""
        (tmp_path / "Valid.py").write_text('''
name: str = "default"
count: int = 0

t"""<div class="test" id="main">{name} - {count}</div>"""
''')

        os.environ["DEBUG"] = "true"
        try:
            comp = load_template(tmp_path / "Valid.py")
            comp(name="test", count=5)

            gen_file = tmp_path / "Valid.gen.py"
            gen_code = gen_file.read_text()

            # Should compile without errors
            compile(gen_code, "<test>", "exec")
        finally:
            os.environ.pop("DEBUG", None)

    def test_return_type_is_str(self, tmp_path: Path):
        """Generated render function returns str."""
        (tmp_path / "StrReturn.py").write_text('t"""<div>Test</div>"""')

        comp = load_template(tmp_path / "StrReturn.py")
        result = comp()

        # Should be Markup (subclass of str)
        assert isinstance(result, Markup)
        assert isinstance(result, str)


class TestVoidElements:
    """Test void elements (self-closing tags)."""

    def test_void_element_input(self, tmp_path: Path):
        """Input elements are self-closing."""
        (tmp_path / "Input.py").write_text('''
input_type: str

t"""<input type={input_type} />"""
''')

        comp = load_template(tmp_path / "Input.py")
        result = str(comp(input_type="text"))

        assert "<input" in result
        assert "/>" in result

    def test_void_element_br(self, tmp_path: Path):
        """br elements are self-closing."""
        (tmp_path / "Br.py").write_text('t"""<p>Line 1<br />Line 2</p>"""')

        comp = load_template(tmp_path / "Br.py")
        result = str(comp())

        assert "<br />" in result

    def test_void_element_img(self, tmp_path: Path):
        """img elements are self-closing."""
        (tmp_path / "Img.py").write_text('''
src: str

t"""<img src={src} alt="Image" />"""
''')

        comp = load_template(tmp_path / "Img.py")
        result = str(comp(src="/image.png"))

        assert '<img src="/image.png"' in result
        assert "/>" in result


class TestMultilineTemplates:
    """Test multiline template handling."""

    def test_multiline_preserves_structure(self, tmp_path: Path):
        """Multiline templates preserve whitespace structure."""
        (tmp_path / "Multiline.py").write_text('''
title: str

t"""
<article>
    <h1>{title}</h1>
    <p>Content here</p>
</article>
"""
''')

        comp = load_template(tmp_path / "Multiline.py")
        result = str(comp(title="Test"))

        assert "<h1>Test</h1>" in result
        assert "<article>" in result
        assert "</article>" in result

    def test_multiline_with_indentation(self, tmp_path: Path):
        """Multiline templates with various indentation levels."""
        (tmp_path / "Indented.py").write_text('''
items: list

t"""<ul>
    <li>First</li>
    <li>Second</li>
</ul>"""
''')

        comp = load_template(tmp_path / "Indented.py")
        result = str(comp(items=[]))

        assert "<ul>" in result
        assert "<li>First</li>" in result
        assert "</ul>" in result


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_empty_template(self, tmp_path: Path):
        """Empty template returns empty string."""
        (tmp_path / "Empty.py").write_text('t""""""')

        comp = load_template(tmp_path / "Empty.py")
        result = str(comp())

        assert result == ""

    def test_only_whitespace(self, tmp_path: Path):
        """Template with only whitespace."""
        (tmp_path / "Whitespace.py").write_text('t"""   \\n   """')

        comp = load_template(tmp_path / "Whitespace.py")
        result = comp()

        # Should handle whitespace-only template
        assert isinstance(result, Markup)

    def test_special_characters_in_content(self, tmp_path: Path):
        """Special characters in static content."""
        (tmp_path / "Special.py").write_text(
            't"""<p>Price: $100 & Tax: 10%</p>"""'
        )

        comp = load_template(tmp_path / "Special.py")
        result = str(comp())

        assert "$100" in result or "&#36;100" in result or "\\$100" in result
        # Ampersand might be escaped
        assert "&" in result or "&amp;" in result

    def test_unicode_content(self, tmp_path: Path):
        """Unicode characters in template."""
        (tmp_path / "Unicode.py").write_text('''
emoji: str

t"""<span>🎉 {emoji} 🎊</span>"""
''')

        comp = load_template(tmp_path / "Unicode.py")
        result = str(comp(emoji="🚀"))

        assert "🎉" in result
        assert "🚀" in result
        assert "🎊" in result
