"""Test context system for dependency injection."""

from pathlib import Path
import pytest

from hyper import context, load_component
from hyper_templates.errors import PropValidationError


class TestContext:
    """Test context storage and retrieval."""

    def test_set_and_get_context(self):
        """Can set and retrieve context."""
        context.set_context({"request": "mock_request", "user": "alice"})

        assert context.get("request") == "mock_request"
        assert context.get("user") == "alice"
        assert context.get("missing") is None
        assert context.get("missing", "default") == "default"

    def test_clear_context(self):
        """Can clear context."""
        context.set_context({"key": "value"})
        context.clear_context()

        assert context.get("key") is None
        assert context.get_context() == {}

    def test_is_dependency(self):
        """Detects framework dependency types."""
        assert context.is_dependency("Request") is True
        assert context.is_dependency("Response") is True
        assert context.is_dependency("Header") is True
        assert context.is_dependency("Cookie") is True

        assert context.is_dependency("str") is False
        assert context.is_dependency("int") is False
        assert context.is_dependency(None) is False


class TestComponentDependencyInjection:
    """Test component dependency injection from context."""

    def test_component_with_regular_props(self, tmp_path: Path):
        """Regular props work as before (no context needed)."""
        component_file = tmp_path / "Button.py"
        component_file.write_text('''
variant: str = "primary"
size: str = "md"

t"""<button class="btn-{variant} btn-{size}">{...}</button>"""
''')

        component = load_component(component_file)
        result = str(component(children=("Click",), variant="success"))

        assert 'class="btn-success btn-md"' in result
        assert "Click" in result

    def test_component_with_dependency_from_context(self, tmp_path: Path):
        """Component can access dependencies from context."""
        component_file = tmp_path / "Nav.py"
        component_file.write_text('''
# Dependency - will be injected from context
request: Request

t"""<nav>Current path: {request.url.path}</nav>"""
''')

        # Set context with a mock request
        class MockRequest:
            class URL:
                path = "/home"
            url = URL()

        context.set_context({"request": MockRequest()})

        try:
            component = load_component(component_file)
            result = str(component())

            assert "Current path: /home" in result
        finally:
            context.clear_context()

    def test_component_with_mixed_props_and_dependencies(self, tmp_path: Path):
        """Component can have both regular props and dependencies."""
        component_file = tmp_path / "UserCard.py"
        component_file.write_text('''
# Regular prop
name: str

# Dependency from context
request: Request

t"""<div>User: {name} | Path: {request.url.path}</div>"""
''')

        class MockRequest:
            class URL:
                path = "/users/123"
            url = URL()

        context.set_context({"request": MockRequest()})

        try:
            component = load_component(component_file)
            result = str(component(name="Alice"))

            assert "User: Alice" in result
            assert "Path: /users/123" in result
        finally:
            context.clear_context()

    def test_missing_required_dependency_raises_error(self, tmp_path: Path):
        """Missing required dependency raises clear error."""
        component_file = tmp_path / "Protected.py"
        component_file.write_text('''
request: Request

t"""<div>{request.method}</div>"""
''')

        # No context set
        context.clear_context()

        component = load_component(component_file)

        with pytest.raises(PropValidationError, match="'Request' dependency not available in request context"):
            component()
