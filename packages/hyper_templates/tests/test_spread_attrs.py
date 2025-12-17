"""Test spread attributes feature."""

from pathlib import Path
import pytest
from hyper import load_component
from hyper_templates._tdom import html as tdom_html


class TestSpreadAttributes:
    """Test forwarding extra attributes to components."""

    def test_extra_attrs_available_in_template(self, tmp_path: Path):
        """Extra attributes are available as __attrs__ in template."""
        (tmp_path / "Button.py").write_text('''
variant: str = "primary"

# Access extra attrs - they're in __attrs__ dict
# Note: tdom converts data-test to data_test (kebab-case to snake_case)
t"""<button class="btn-{variant}" data-extra={len(__attrs__)}>
  {__attrs__.get('data_test', 'none')} - {...}
</button>"""
''')

        Button = load_component(tmp_path / "Button.py")

        result = tdom_html(t'''<{Button} variant="secondary" data-test="foo" aria-label="Click">
          Click Me
        </{Button}>''')

        # Extra attrs (data-test, aria-label) are passed but not declared as props
        # tdom converts them to snake_case: data_test, aria_label
        assert 'data-extra="2"' in str(result)
        assert "foo" in str(result)

    def test_component_without_spread_doesnt_forward(self, tmp_path: Path):
        """By default, extra attrs don't automatically appear in output."""
        (tmp_path / "Button.py").write_text('''
t"""<button>{...}</button>"""
''')

        Button = load_component(tmp_path / "Button.py")

        result = tdom_html(t'<{Button} data-test="foo" type="submit">Click</{Button}>')

        # The button element doesn't have these attributes
        # They're available in __attrs__ but not automatically spread
        assert "data-test" not in str(result)
        assert "type" not in str(result)
        assert "Click" in str(result)

    def test_manual_attribute_forwarding(self, tmp_path: Path):
        """Components can manually forward specific extra attributes."""
        (tmp_path / "Button.py").write_text('''
# Note: Access extra attrs with snake_case names (data_test not data-test)
t"""<button type={__attrs__.get('type', 'button')}
         data-test={__attrs__.get('data_test', '')}>
  {...}
</button>"""
''')

        Button = load_component(tmp_path / "Button.py")

        result = tdom_html(t'<{Button} type="submit" data-test="btn">Click</{Button}>')

        assert 'type="submit"' in str(result)
        assert 'data-test="btn"' in str(result)

    def test_class_merging_pattern(self, tmp_path: Path):
        """Components can merge classes from props and extra attrs."""
        (tmp_path / "Button.py").write_text('''
variant: str = "primary"

# Merge classes inline in t-string
t"""<button class={"btn btn-" + variant + " " + __attrs__.get('_class', '')}>{...}</button>"""
''')

        Button = load_component(tmp_path / "Button.py")

        result = tdom_html(t'<{Button} variant="success" _class="mt-4 rounded">Click</{Button}>')

        result_str = str(result)
        assert "btn" in result_str
        assert "btn-success" in result_str
        assert "mt-4" in result_str
        assert "rounded" in result_str
