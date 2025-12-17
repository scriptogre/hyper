"""Shared test fixtures for hyper-templates."""

import pytest
from pathlib import Path


@pytest.fixture
def component_dir(tmp_path: Path) -> Path:
    """Create a temporary directory for test components."""
    return tmp_path


@pytest.fixture
def simple_layout(tmp_path: Path) -> Path:
    """Create a simple layout component."""
    layout_path = tmp_path / "Layout.py"
    layout_path.write_text('''
title: str = "My Site"

t"""
<!doctype html>
<html>
<head><title>{title}</title></head>
<body>{...}</body>
</html>
"""
''')
    return layout_path


@pytest.fixture
def simple_button(tmp_path: Path) -> Path:
    """Create a simple button component."""
    button_path = tmp_path / "Button.py"
    button_path.write_text('''
t"""<button>{...}</button>"""
''')
    return button_path


@pytest.fixture
def component_with_required_prop(tmp_path: Path) -> Path:
    """Create a component with a required prop."""
    path = tmp_path / "Required.py"
    path.write_text('''
name: str

t"""<h1>Hello, {name}!</h1>"""
''')
    return path
