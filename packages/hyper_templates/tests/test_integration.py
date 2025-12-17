"""Integration tests for the complete template workflow."""

import pytest
from pathlib import Path

from hyper import render, load_component


class TestBasicRendering:
    """Tests for basic render functionality."""

    def test_render_string(self):
        """render() passes strings through."""
        result = render("Hello, World!")
        assert result == "Hello, World!"


class TestLayoutIntegration:
    """Integration tests with layout components.

    Note: These tests are placeholders for when tdom integration is complete.
    Currently they just verify the component loading works.
    """

    def test_load_layout_component(self, tmp_path: Path):
        """Load a layout component and verify props."""
        (tmp_path / "Layout.py").write_text('''
title: str = "My Site"

t"""
<!doctype html>
<html>
<head><title>{title}</title></head>
<body>{...}</body>
</html>
"""
''')

        Layout = load_component(tmp_path / "Layout.py")

        assert Layout.name == "Layout"
        assert "title" in Layout.props
        assert Layout.props["title"].default == "My Site"

    def test_nested_components(self, tmp_path: Path):
        """Multiple components can be loaded."""
        (tmp_path / "Card.py").write_text('''
t"""<div class="card">{...}</div>"""
''')
        (tmp_path / "Button.py").write_text('''
label: str = "Click"

t"""<button>{label}</button>"""
''')

        Card = load_component(tmp_path / "Card.py")
        Button = load_component(tmp_path / "Button.py")

        assert Card.name == "Card"
        assert Button.name == "Button"
        assert Button.props["label"].default == "Click"


# TODO: Add these tests once tdom integration is complete
#
# class TestFullRenderWorkflow:
#     """Tests for complete render with t-strings."""
#
#     def test_full_layout_render(self, tmp_path: Path):
#         \"\"\"Complete layout + content rendering.\"\"\"
#         (tmp_path / "Layout.py").write_text('''
# title: str = "My Site"
# t\"\"\"<!doctype html><html><title>{title}</title><body>{...}</body></html>\"\"\"
# ''')
#         Layout = load_component(tmp_path / "Layout.py")
#
#         result = render(t\"\"\"
#         <{Layout} title="Home">
#             <h1>Welcome!</h1>
#         </{Layout}>
#         \"\"\")
#
#         assert "<title>Home</title>" in result
#         assert "<h1>Welcome!</h1>" in result
