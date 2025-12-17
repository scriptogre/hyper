"""Context system for passing request/framework state to components."""

from contextvars import ContextVar
from typing import Any

# Global context variable (thread-safe and async-safe)
_render_context: ContextVar[dict[str, Any] | None] = ContextVar('render_context', default=None)


def get_context() -> dict[str, Any]:
    """Get the current render context."""
    return _render_context.get() or {}


def set_context(context: dict[str, Any]) -> None:
    """Set the render context (called by framework)."""
    _render_context.set(context)


def clear_context() -> None:
    """Clear the render context."""
    _render_context.set(None)


def get(key: str, default: Any = None) -> Any:
    """Get a value from context by key."""
    return get_context().get(key, default)


# Common dependency types that should be resolved from context
_DEPENDENCY_TYPES = {
    'Request',
    'Response',
    'Header',
    'Cookie',
    'Form',
    'Body',
    'File',
    'UploadFile',
}


def is_dependency(type_name: str | None) -> bool:
    """Check if a type hint represents a framework dependency."""
    if not type_name:
        return False
    # Handle Annotated types
    if type_name.startswith('Annotated['):
        return True
    return type_name in _DEPENDENCY_TYPES
