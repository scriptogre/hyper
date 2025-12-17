"""Template exceptions with contextual error messages."""

from pathlib import Path


class TemplateError(Exception):
    """Base exception for all template errors."""

    def __init__(self, message: str, path: Path | None = None):
        self.path = path
        super().__init__(f"{message}\n\n  File: {path}" if path else message)


class PropValidationError(TemplateError):
    def __init__(
        self,
        message: str,
        path: Path | None = None,
        component_name: str | None = None,
        props: dict | None = None,
    ):
        self.component_name = component_name
        self.props = props or {}

        full_message = message
        if path:
            full_message += f"\n\n  File: {path}"

        if component_name and props:
            full_message += f"\n\n  Component {component_name} requires:"
            for name, prop in props.items():
                type_name = prop.type_hint.__name__ if prop.type_hint else "Any"
                if prop.has_default:
                    full_message += f"\n    - {name}: {type_name} = {prop.default!r}"
                else:
                    full_message += f"\n    - {name}: {type_name} (required)"

        Exception.__init__(self, full_message)
        self.path = path


class ComponentNotFoundError(TemplateError):
    pass


class SlotError(TemplateError):
    pass


class ComponentCompileError(TemplateError):
    """Error during component compilation."""

    def __init__(
        self,
        message: str,
        path: Path | None = None,
        line: int | None = None,
        original_error: Exception | None = None,
    ):
        self.line = line
        self.original_error = original_error

        full_message = message
        if path:
            location = f"File: {path}"
            if line is not None:
                location += f", line {line}"
            full_message += f"\n\n  {location}"

        if original_error:
            full_message += f"\n\n  Original error: {type(original_error).__name__}: {original_error}"

        Exception.__init__(self, full_message)
        self.path = path
