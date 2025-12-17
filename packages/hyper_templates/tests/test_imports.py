"""Test magic import system for components."""

from pathlib import Path
import sys
import pytest


class TestComponentImports:
    """Test importing components with from X import Y syntax."""

    def test_import_component_from_package(self, tmp_path: Path):
        """Components can be imported from packages using auto-installed import hook."""
        # Create package structure
        components_dir = tmp_path / "components"
        components_dir.mkdir()

        # Create Button component
        (components_dir / "Button.py").write_text('''
variant: str = "primary"
t"""<button class="btn-{variant}">{...}</button>"""
''')

        # Create empty __init__.py (import hook handles the rest)
        (components_dir / "__init__.py").write_text('')

        # Add to sys.path so we can import
        sys.path.insert(0, str(tmp_path))

        try:
            # Import the component
            from components import Button

            # Verify it's a callable component
            assert callable(Button)

            # Verify it works
            from hyper_templates._tdom import html as tdom_html
            result = tdom_html(t"""<{Button} variant="secondary">Click</{Button}>""")
            assert 'class="btn-secondary"' in str(result)
            assert "Click" in str(result)

        finally:
            # Cleanup
            sys.path.remove(str(tmp_path))
            if "components" in sys.modules:
                del sys.modules["components"]

    def test_import_multiple_components(self, tmp_path: Path):
        """Multiple components can be imported from same package."""
        components_dir = tmp_path / "components"
        components_dir.mkdir()

        (components_dir / "Button.py").write_text('t"""<button>{...}</button>"""')
        (components_dir / "Card.py").write_text('t"""<div class="card">{...}</div>"""')

        # Create empty __init__.py (import hook handles the rest)
        (components_dir / "__init__.py").write_text('')

        sys.path.insert(0, str(tmp_path))

        try:
            from components import Button, Card

            assert callable(Button)
            assert callable(Card)

        finally:
            sys.path.remove(str(tmp_path))
            if "components" in sys.modules:
                del sys.modules["components"]

    def test_import_nonexistent_component_raises_error(self, tmp_path: Path):
        """Importing non-existent component raises ImportError."""
        components_dir = tmp_path / "components"
        components_dir.mkdir()

        # Create empty __init__.py (import hook handles the rest)
        (components_dir / "__init__.py").write_text('')

        sys.path.insert(0, str(tmp_path))

        try:
            with pytest.raises(ImportError, match="cannot import name 'NonExistent'"):
                from components import NonExistent

        finally:
            sys.path.remove(str(tmp_path))
            if "components" in sys.modules:
                del sys.modules["components"]

    def test_nested_components_via_imports(self, tmp_path: Path):
        """Components imported via auto-installed hook work in nested structures."""
        components_dir = tmp_path / "components"
        components_dir.mkdir()

        (components_dir / "Layout.py").write_text('''
title: str = "Site"
t"""<html><head><title>{title}</title></head><body>{...}</body></html>"""
''')

        (components_dir / "Card.py").write_text('''
t"""<div class="card">{...}</div>"""
''')

        # Create empty __init__.py (import hook handles the rest)
        (components_dir / "__init__.py").write_text('')

        sys.path.insert(0, str(tmp_path))

        try:
            from components import Layout, Card
            from hyper_templates._tdom import html as tdom_html

            result = tdom_html(t"""
<{Layout} title="Home">
  <{Card}>Content</{Card}>
</{Layout}>
""")

            assert "<title>Home</title>" in str(result)
            assert '<div class="card">Content</div>' in str(result)

        finally:
            sys.path.remove(str(tmp_path))
            if "components" in sys.modules:
                del sys.modules["components"]
