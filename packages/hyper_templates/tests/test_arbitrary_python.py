"""Test arbitrary Python code in components."""

from pathlib import Path
import pytest
from hyper import load_component
from hyper_templates._tdom import html as tdom_html


class TestArbitraryPython:
    """Test that components can run arbitrary Python code before the template."""

    def test_computed_variables(self, tmp_path: Path):
        """Components can compute variables from props."""
        (tmp_path / "Button.py").write_text('''
variant: str = "primary"
size: str = "md"

# Compute derived values
classes = f"btn btn-{variant} btn-{size}"
is_large = size == "lg"

t"""<button class={classes} data-large={is_large}>{...}</button>"""
''')

        Button = load_component(tmp_path / "Button.py")

        result = tdom_html(t'<{Button} variant="secondary" size="lg">Click</{Button}>')
        result_str = str(result)
        assert "btn-secondary" in result_str
        assert "btn-lg" in result_str
        assert "data-large" in result_str  # Boolean True renders as attribute without value

    def test_conditional_logic(self, tmp_path: Path):
        """Components can use conditional logic."""
        (tmp_path / "Alert.py").write_text('''
type: str = "info"

# Conditional icon based on type
if type == "error":
    icon = "❌"
elif type == "warning":
    icon = "⚠️"
elif type == "success":
    icon = "✅"
else:
    icon = "ℹ️"

t"""<div class="alert alert-{type}">{icon} {...}</div>"""
''')

        Alert = load_component(tmp_path / "Alert.py")

        result_error = tdom_html(t'<{Alert} type="error">Error message</{Alert}>')
        assert "❌" in str(result_error)

        result_success = tdom_html(t'<{Alert} type="success">Success!</{Alert}>')
        assert "✅" in str(result_success)

    def test_string_manipulation(self, tmp_path: Path):
        """Components can manipulate strings."""
        (tmp_path / "Heading.py").write_text('''
title: str = ""

# Transform title
title_upper = title.upper()
title_slug = title.lower().replace(" ", "-")

t"""<h1 id={title_slug}>{title_upper}</h1>"""
''')

        Heading = load_component(tmp_path / "Heading.py")

        result = tdom_html(t'<{Heading} title="Hello World" />')
        assert "HELLO WORLD" in str(result)
        assert 'id="hello-world"' in str(result)

    def test_list_operations(self, tmp_path: Path):
        """Components can work with lists."""
        (tmp_path / "ClassList.py").write_text('''
base_class: str = "btn"
extra_classes: str = ""

# Combine classes
all_classes = [base_class]
if extra_classes:
    all_classes.extend(extra_classes.split())
classes_str = " ".join(all_classes)

t"""<button class={classes_str}>{...}</button>"""
''')

        ClassList = load_component(tmp_path / "ClassList.py")

        result = tdom_html(t'<{ClassList} extra_classes="rounded shadow">Click</{ClassList}>')
        assert "btn" in str(result)
        assert "rounded" in str(result)
        assert "shadow" in str(result)

    def test_dictionary_operations(self, tmp_path: Path):
        """Components can work with dictionaries."""
        (tmp_path / "DataAttrs.py").write_text('''
user_id: str = ""
status: str = "active"

# Build data attributes
data_attrs = {
    "user-id": user_id,
    "status": status,
    "timestamp": "2025-01-01"
}

t"""<div data-user-id={data_attrs["user-id"]}
         data-status={data_attrs["status"]}>
  {...}
</div>"""
''')

        DataAttrs = load_component(tmp_path / "DataAttrs.py")

        result = tdom_html(t'<{DataAttrs} user_id="123">Content</{DataAttrs}>')
        assert 'data-user-id="123"' in str(result)
        assert 'data-status="active"' in str(result)

    def test_accessing_extra_attrs_in_code(self, tmp_path: Path):
        """Components can use __attrs__ in arbitrary Python code."""
        (tmp_path / "SmartButton.py").write_text('''
variant: str = "primary"

# Check for extra classes and merge them
extra_class = __attrs__.get("_class", "")
all_classes = f"btn btn-{variant} {extra_class}".strip()

# Check if disabled
is_disabled = __attrs__.get("disabled", False)

t"""<button class={all_classes} disabled={is_disabled}>{...}</button>"""
''')

        SmartButton = load_component(tmp_path / "SmartButton.py")

        result = tdom_html(t'<{SmartButton} variant="success" _class="rounded" disabled={True}>Save</{SmartButton}>')
        result_str = str(result)
        assert "btn-success" in result_str
        assert "rounded" in result_str
        assert "disabled" in result_str  # Boolean attribute renders without value

    def test_multiline_computations(self, tmp_path: Path):
        """Components can have multi-line Python code blocks."""
        (tmp_path / "Card.py").write_text('''
title: str = ""
count: int = 0

# Multiple lines of computation
title_formatted = title.strip().title()
count_display = str(count)
if count > 1000:
    count_display = f"{count / 1000:.1f}k"
elif count > 1000000:
    count_display = f"{count / 1000000:.1f}m"

badge_color = "success" if count > 100 else "warning"

t"""<div class="card">
  <h3>{title_formatted}</h3>
  <span class="badge badge-{badge_color}">{count_display}</span>
  <div>{...}</div>
</div>"""
''')

        Card = load_component(tmp_path / "Card.py")

        result = tdom_html(t'<{Card} title="user stats" count={1500}>Details</{Card}>')
        assert "User Stats" in str(result)
        assert "1.5k" in str(result)
        assert "badge-success" in str(result)
