"""Test magic import system for templates."""

from pathlib import Path
import sys
import pytest


def cleanup_modules():
    """Remove all 'templates' related modules from sys.modules and meta_path finders."""
    # Remove modules
    to_remove = [k for k in sys.modules if k == "templates" or k.startswith("templates.")]
    for k in to_remove:
        del sys.modules[k]

    # Remove ALL template finders from meta_path (not just those we track)
    from hyper.templates import _enable_templates_instance, _TemplateFinder
    # Clear the enabled modules set
    _enable_templates_instance._enabled_modules.discard("templates")
    # Remove finders for templates package
    if "templates" in _enable_templates_instance._finders:
        _enable_templates_instance._finders.pop("templates")

    # Remove any _TemplateFinder instances for templates
    to_remove_finders = [f for f in sys.meta_path if isinstance(f, _TemplateFinder) and f.package_name == "templates"]
    for f in to_remove_finders:
        sys.meta_path.remove(f)


@pytest.fixture(autouse=True)
def clean_between_tests():
    """Clean up modules before and after each test."""
    cleanup_modules()
    yield
    cleanup_modules()


class TestTemplateImports:
    """Test importing templates with from X import Y syntax."""

    def test_import_template_from_package(self, tmp_path: Path):
        """Templates can be imported from packages using enable_templates."""
        # Create package structure
        templates_dir = tmp_path / "templates"
        templates_dir.mkdir()

        # Create Button template
        (templates_dir / "Button.py").write_text('''
variant: str = "primary"
t"""<button class="btn-{variant}">{...}</button>"""
''')

        # Enable template imports for this package
        (templates_dir / "__init__.py").write_text(
            "from hyper import enable_templates"
        )

        # Add to sys.path so we can import
        sys.path.insert(0, str(tmp_path))

        try:
            # Import the template
            from templates import Button

            # Verify it's a callable
            assert callable(Button)

            # Verify it works
            from hyper.templates._tdom import html as tdom_html

            result = tdom_html(t"""<{Button} variant="secondary">Click</{Button}>""")
            assert 'class="btn-secondary"' in str(result)
            assert "Click" in str(result)

        finally:
            sys.path.remove(str(tmp_path))
            cleanup_modules()

    def test_import_multiple_templates(self, tmp_path: Path):
        """Multiple templates can be imported from same package."""
        templates_dir = tmp_path / "templates"
        templates_dir.mkdir()

        (templates_dir / "Button.py").write_text('t"""<button>{...}</button>"""')
        (templates_dir / "Card.py").write_text('t"""<div class="card">{...}</div>"""')

        # Enable template imports for this package
        (templates_dir / "__init__.py").write_text(
            "from hyper import enable_templates"
        )

        sys.path.insert(0, str(tmp_path))

        try:
            from templates import Button, Card

            assert callable(Button)
            assert callable(Card)

        finally:
            sys.path.remove(str(tmp_path))
            cleanup_modules()

    def test_import_nonexistent_template_raises_error(self, tmp_path: Path):
        """Importing non-existent template raises ImportError."""
        templates_dir = tmp_path / "templates"
        templates_dir.mkdir()

        # Enable template imports for this package
        (templates_dir / "__init__.py").write_text(
            "from hyper import enable_templates"
        )

        sys.path.insert(0, str(tmp_path))

        try:
            with pytest.raises(ImportError):
                from templates import NonExistent  # noqa: F401

        finally:
            sys.path.remove(str(tmp_path))
            cleanup_modules()

    def test_nested_templates_via_imports(self, tmp_path: Path):
        """Templates imported via enable_templates work in nested structures."""
        templates_dir = tmp_path / "templates"
        templates_dir.mkdir()

        (templates_dir / "Layout.py").write_text('''
title: str = "Site"
t"""<html><head><title>{title}</title></head><body>{...}</body></html>"""
''')

        (templates_dir / "Card.py").write_text('''
t"""<div class="card">{...}</div>"""
''')

        # Enable template imports for this package
        (templates_dir / "__init__.py").write_text(
            "from hyper import enable_templates"
        )

        sys.path.insert(0, str(tmp_path))

        try:
            from templates import Layout, Card
            from hyper.templates._tdom import html as tdom_html

            result = tdom_html(t"""
<{Layout} title="Home">
  <{Card}>Content</{Card}>
</{Layout}>
""")

            assert "<title>Home</title>" in str(result)
            assert '<div class="card">Content</div>' in str(result)

        finally:
            sys.path.remove(str(tmp_path))
            cleanup_modules()
