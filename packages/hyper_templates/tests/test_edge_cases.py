"""Edge case tests for robustness and security."""

import pytest
from pathlib import Path

from hyper import render, load_component
from hyper_templates.component import Component
from hyper_templates.errors import ComponentNotFoundError, PropValidationError
from hyper_templates._tdom.nodes import Node


class TestComponentEdgeCases:
    """Tests for component edge cases and error handling."""

    def test_component_without_template(self, tmp_path: Path):
        """Component file with no t-string returns empty."""
        (tmp_path / "Empty.py").write_text("""
title: str = "Hello"
# Just props, no template
""")

        component = load_component(tmp_path / "Empty.py")
        result = component()

        assert str(result) == ""

    def test_component_with_invalid_template_syntax(self, tmp_path: Path):
        """Component with invalid template syntax raises ComponentCompileError at load time."""
        from hyper import ComponentCompileError

        (tmp_path / "Invalid.py").write_text("""
t\"\"\"<div>{invalid syntax that will fail}\"\"\"
""")

        # New behavior: fail loudly at load time with clear error
        with pytest.raises(ComponentCompileError) as exc_info:
            load_component(tmp_path / "Invalid.py")

        assert "Invalid.py" in str(exc_info.value)

    def test_component_with_undefined_variable(self, tmp_path: Path):
        """Template referencing undefined variable raises NameError at render time."""
        (tmp_path / "Undefined.py").write_text("""
t\"\"\"<div>{undefined_variable}</div>\"\"\"
""")

        component = load_component(tmp_path / "Undefined.py")

        # New behavior: fail at render time with proper stack trace
        with pytest.raises(NameError) as exc_info:
            component()

        assert "undefined_variable" in str(exc_info.value)

    def test_component_with_no_props_no_slots(self, tmp_path: Path):
        """Simple component with just static HTML."""
        (tmp_path / "Static.py").write_text("""
t\"\"\"<div class="static">Hello World</div>\"\"\"
""")

        component = load_component(tmp_path / "Static.py")
        result = component()

        assert "<div" in result
        assert "Hello World" in result

    def test_component_with_empty_children(self, tmp_path: Path):
        """Component with slot but no children provided."""
        (tmp_path / "Wrapper.py").write_text("""
t\"\"\"<div class="wrapper">{...}</div>\"\"\"
""")

        component = load_component(tmp_path / "Wrapper.py")
        result = component()  # No children

        assert '<div class="wrapper"></div>' in result

    def test_component_with_multiple_children(self, tmp_path: Path):
        """Component receives multiple children."""
        (tmp_path / "List.py").write_text("""
t\"\"\"<ul>{...}</ul>\"\"\"
""")

        component = load_component(tmp_path / "List.py")
        result = component(children=(
            "<li>Item 1</li>",
            "<li>Item 2</li>",
            "<li>Item 3</li>",
        ))

        assert "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>" in result

    def test_html_escaping_in_props(self, tmp_path: Path):
        """HTML in props is escaped to prevent XSS."""
        (tmp_path / "SafeText.py").write_text("""
text: str = ""

t\"\"\"<div>{text}</div>\"\"\"
""")

        component = load_component(tmp_path / "SafeText.py")
        result = component(text='<script>alert("xss")</script>')

        # HTML should be escaped
        assert "&lt;script&gt;" in result
        assert "<script>" not in result

    def test_children_html_not_escaped(self, tmp_path: Path):
        """Children HTML is not escaped (trusted content)."""
        (tmp_path / "Container.py").write_text("""
t\"\"\"<div>{...}</div>\"\"\"
""")

        component = load_component(tmp_path / "Container.py")
        result = component(children=("<strong>Bold</strong>",))

        # Children should not be escaped
        assert "<strong>Bold</strong>" in result
        assert "&lt;strong&gt;" not in result

    def test_special_characters_in_props(self, tmp_path: Path):
        """Props with special characters are handled correctly."""
        (tmp_path / "Special.py").write_text("""
text: str = ""

t\"\"\"<div>{text}</div>\"\"\"
""")

        component = load_component(tmp_path / "Special.py")
        result = component(text='Quotes: " & \' < >')

        # Should be escaped
        assert "&quot;" in result or "&#34;" in result
        assert "&amp;" in result
        assert "&lt;" in result
        assert "&gt;" in result

    def test_numeric_props(self, tmp_path: Path):
        """Numeric props are rendered correctly."""
        (tmp_path / "Counter.py").write_text("""
count: int = 0
price: float = 0.0

t\"\"\"<div>Count: {count}, Price: {price}</div>\"\"\"
""")

        component = load_component(tmp_path / "Counter.py")
        result = component(count=42, price=19.99)

        assert "Count: 42" in result
        assert "Price: 19.99" in result

    def test_boolean_props(self, tmp_path: Path):
        """Boolean props are rendered correctly."""
        (tmp_path / "Flag.py").write_text("""
enabled: bool = False

t\"\"\"<div>Enabled: {enabled}</div>\"\"\"
""")

        component = load_component(tmp_path / "Flag.py")

        result_true = component(enabled=True)
        assert "Enabled: True" in result_true

        result_false = component(enabled=False)
        assert "Enabled: False" in result_false

    def test_none_prop_value(self, tmp_path: Path):
        """None as a prop value is rendered as empty (tdom behavior)."""
        (tmp_path / "Optional.py").write_text("""
value: str = "default"

t\"\"\"<div>{value}</div>\"\"\"
""")

        component = load_component(tmp_path / "Optional.py")
        # tdom treats None as falsy and renders it as empty
        result = component(value=None)

        assert "<div></div>" in result


class TestRenderEdgeCases:
    """Tests for render() function edge cases."""

    def test_render_with_int(self):
        """render() with int returns string."""
        result = render(123)
        assert str(result) == "123"

    def test_render_with_float(self):
        """render() with float returns string."""
        result = render(45.67)
        assert str(result) == "45.67"

    def test_render_with_none(self):
        """render() with None returns 'None'."""
        result = render(None)
        assert str(result) == "None"

    def test_render_with_bool(self):
        """render() with bool returns string."""
        assert render(True) == "True"
        assert render(False) == "False"

    def test_render_with_empty_string(self):
        """render() with empty string returns empty."""
        result = render("")
        assert str(result) == ""

    def test_render_with_html_string(self):
        """render() passes HTML strings through."""
        html = "<div>Test</div>"
        result = render(html)
        assert result == html


class TestErrorMessages:
    """Tests for error message quality and helpfulness."""

    def test_missing_required_prop_error_message(self, tmp_path: Path):
        """Error message for missing prop is helpful."""
        (tmp_path / "Required.py").write_text("""
name: str
age: int = 0

t\"\"\"<div>{name}</div>\"\"\"
""")

        component = load_component(tmp_path / "Required.py")

        with pytest.raises(PropValidationError) as exc:
            component()

        error_msg = str(exc.value)
        assert "name" in error_msg
        assert "Required.py" in error_msg

    def test_wrong_type_error_message(self, tmp_path: Path):
        """Error message for type mismatch is helpful."""
        (tmp_path / "Typed.py").write_text("""
count: int = 0

t\"\"\"<div>{count}</div>\"\"\"
""")

        component = load_component(tmp_path / "Typed.py")

        with pytest.raises(PropValidationError) as exc:
            component(count="not a number")

        error_msg = str(exc.value)
        assert "count" in error_msg
        assert "int" in error_msg
        assert "str" in error_msg

    def test_component_not_found_error_message(self):
        """Error message for missing file is helpful."""
        with pytest.raises(ComponentNotFoundError) as exc:
            load_component(Path("/nonexistent/path/Component.py"))

        error_msg = str(exc.value)
        assert "Component.py" in error_msg or "not found" in error_msg.lower()


class TestNestedComponents:
    """Tests for complex nested component scenarios."""

    def test_deeply_nested_components(self, tmp_path: Path):
        """Multiple levels of nesting work correctly."""
        (tmp_path / "Level1.py").write_text('t"""<div class="l1">{...}</div>"""')
        (tmp_path / "Level2.py").write_text('t"""<div class="l2">{...}</div>"""')
        (tmp_path / "Level3.py").write_text('t"""<div class="l3">Content</div>"""')

        L1 = load_component(tmp_path / "Level1.py")
        L2 = load_component(tmp_path / "Level2.py")
        L3 = load_component(tmp_path / "Level3.py")

        l3_html = L3()
        l2_html = L2(children=(l3_html,))
        result = L1(children=(l2_html,))

        assert 'class="l1"' in result
        assert 'class="l2"' in result
        assert 'class="l3"' in result
        assert "Content" in result

    def test_component_reuse(self, tmp_path: Path):
        """Same component can be used multiple times."""
        (tmp_path / "Item.py").write_text("""
text: str = ""

t\"\"\"<li>{text}</li>\"\"\"
""")

        Item = load_component(tmp_path / "Item.py")

        item1 = Item(text="First")
        item2 = Item(text="Second")
        item3 = Item(text="Third")

        assert "<li>First</li>" in item1
        assert "<li>Second</li>" in item2
        assert "<li>Third</li>" in item3

    def test_mixed_static_and_dynamic_children(self, tmp_path: Path):
        """Children can mix static HTML and component output."""
        (tmp_path / "Container.py").write_text('t"""<div>{...}</div>"""')
        (tmp_path / "Bold.py").write_text("""
text: str = ""
t\"\"\"<strong>{text}</strong>\"\"\"
""")

        Container = load_component(tmp_path / "Container.py")
        Bold = load_component(tmp_path / "Bold.py")

        bold_html = Bold(text="Important")
        result = Container(children=(
            "<p>Start</p>",
            bold_html,
            "<p>End</p>",
        ))

        assert "<p>Start</p>" in result
        assert "<strong>Important</strong>" in result
        assert "<p>End</p>" in result


class TestTemplateExtraction:
    """Tests for template string extraction edge cases."""

    def test_multiple_tstrings_uses_first(self, tmp_path: Path):
        """When multiple t-strings exist, first one is used."""
        (tmp_path / "Multiple.py").write_text("""
t\"\"\"<div>First</div>\"\"\"
t\"\"\"<div>Second</div>\"\"\"
""")

        component = load_component(tmp_path / "Multiple.py")
        result = component()

        assert "First" in result

    def test_tstring_with_single_quotes(self, tmp_path: Path):
        """T-string with single quotes is extracted."""
        (tmp_path / "Single.py").write_text("""
t'''<div>Single quotes</div>'''
""")

        component = load_component(tmp_path / "Single.py")
        result = component()

        assert "Single quotes" in result

    def test_tstring_with_escaped_quotes(self, tmp_path: Path):
        """T-string with escaped quotes inside."""
        (tmp_path / "Escaped.py").write_text("""
text: str = "value"

t\"\"\"<div data-value=\\"{text}\\">Test</div>\"\"\"
""")

        component = load_component(tmp_path / "Escaped.py")
        result = component(text="hello")

        # Should handle escaped quotes in template
        assert "Test" in result


class TestRealWorldScenarios:
    """Tests for realistic usage patterns."""

    def test_blog_post_layout(self, tmp_path: Path):
        """Complete blog post with layout, metadata, and content."""
        (tmp_path / "BlogLayout.py").write_text("""
title: str = "Blog"

t\"\"\"
<!DOCTYPE html>
<html>
<head><title>{title}</title></head>
<body>
  <main>{...}</main>
</body>
</html>
\"\"\"
""")

        (tmp_path / "Article.py").write_text("""
title: str = ""
author: str = ""
date: str = ""

t\"\"\"
<article>
  <h1>{title}</h1>
  <div class="meta">By {author} on {date}</div>
  <div class="content">{...}</div>
</article>
\"\"\"
""")

        Layout = load_component(tmp_path / "BlogLayout.py")
        Article = load_component(tmp_path / "Article.py")

        article_html = Article(
            title="My First Post",
            author="Alice",
            date="2025-01-01",
            children=("<p>This is the content.</p>",)
        )

        result = Layout(title="Alice's Blog", children=(article_html,))

        assert "<!DOCTYPE html>" in result
        assert "<title>Alice&#39;s Blog</title>" in result or "<title>Alice's Blog</title>" in result
        assert "<h1>My First Post</h1>" in result
        assert "By Alice on 2025-01-01" in result
        assert "<p>This is the content.</p>" in result

    def test_dashboard_with_multiple_cards(self, tmp_path: Path):
        """Dashboard with multiple card components."""
        (tmp_path / "Dashboard.py").write_text("""
t\"\"\"
<div class="dashboard">
  {... }
</div>
\"\"\"
""")

        (tmp_path / "Card.py").write_text("""
title: str = ""
value: str = ""

t\"\"\"
<div class="card">
  <h3>{title}</h3>
  <p class="value">{value}</p>
</div>
\"\"\"
""")

        Dashboard = load_component(tmp_path / "Dashboard.py")
        Card = load_component(tmp_path / "Card.py")

        cards = [
            Card(title="Users", value="1,234"),
            Card(title="Revenue", value="$56,789"),
            Card(title="Growth", value="+12%"),
        ]

        result = Dashboard(children=tuple(cards))

        assert 'class="dashboard"' in result
        assert "Users" in result
        assert "1,234" in result
        assert "Revenue" in result
        assert "$56,789" in result
        assert "Growth" in result
        assert "+12%" in result

    def test_form_with_validation_errors(self, tmp_path: Path):
        """Form component with optional error message."""
        (tmp_path / "FormField.py").write_text("""
label: str = ""
error: str = ""

t\"\"\"
<div class="field">
  <label>{label}</label>
  {... }
  <span class="error">{error}</span>
</div>
\"\"\"
""")

        Field = load_component(tmp_path / "FormField.py")

        # Without error
        result_ok = Field(
            label="Email",
            error="",
            children=('<input type="email" />',)
        )
        assert "<label>Email</label>" in result_ok
        assert '<input type="email" />' in result_ok

        # With error
        result_err = Field(
            label="Email",
            error="Invalid email address",
            children=('<input type="email" class="invalid" />',)
        )
        assert "Invalid email address" in result_err
