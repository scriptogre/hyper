"""Test integration with tdom's component syntax."""

from pathlib import Path
import pytest
from hyper import load_component
from hyper_templates._tdom import html as tdom_html


class TestTdomComponentSyntax:
    """Test that Components work with tdom's <{Component}> syntax."""

    def test_simple_component_in_tstring(self, tmp_path: Path):
        """Component can be used with <{Component}> syntax."""
        (tmp_path / "Button.py").write_text('''
t"""<button>{...}</button>"""
''')
        Button = load_component(tmp_path / "Button.py")
        result = tdom_html(t'''<{Button}>Click me</{Button}>''')
        assert str(result) == "<button>Click me</button>"

    def test_component_with_props_in_tstring(self, tmp_path: Path):
        """Component props work with tdom syntax."""
        (tmp_path / "Button.py").write_text('''
variant: str = "primary"
t"""<button class="{variant}">{...}</button>"""
''')
        Button = load_component(tmp_path / "Button.py")
        result = tdom_html(t'''<{Button} variant="secondary">Click</{Button}>''')
        assert str(result) == '<button class="secondary">Click</button>'

    def test_nested_components_in_tstring(self, tmp_path: Path):
        """Nested components work."""
        (tmp_path / "Card.py").write_text('''
t"""<div class="card">{...}</div>"""
''')
        (tmp_path / "Layout.py").write_text('''
t"""<main>{...}</main>"""
''')
        Card = load_component(tmp_path / "Card.py")
        Layout = load_component(tmp_path / "Layout.py")

        result = tdom_html(t'''
<{Layout}>
  <{Card}>Hello</{Card}>
</{Layout}>
''')
        assert "<main>" in str(result)
        assert '<div class="card">Hello</div>' in str(result)
