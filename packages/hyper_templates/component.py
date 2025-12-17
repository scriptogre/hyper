"""Component system: Python files as reusable templates."""

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable

from hyper_templates._tdom import html as tdom_html
from hyper_templates._tdom.nodes import Fragment, Node
from hyper_templates.errors import ComponentNotFoundError, PropValidationError
from hyper_templates.loader import Prop, extract_props
from hyper_templates import context


@dataclass(frozen=True)
class Component:
    """Callable template loaded from a Python file."""

    # Component identity
    path: Path
    name: str
    code: str = ""

    # Internal state
    _props: dict[str, Prop] = field(default_factory=dict, compare=False, hash=False)
    _render: Callable | None = field(default=None, repr=False, compare=False, hash=False)
    _render_code: str = field(default="", repr=False, compare=False, hash=False)

    @property
    def props(self) -> dict[str, Prop]:
        """Get props as a dict (for API compatibility)."""
        return self._props

    def __call__(self, children: tuple = (), **kwargs: Any) -> Node:
        """Render component with props and children."""
        if self._render is None:
            # Fallback for components without compiled function
            return Fragment(children=[])

        # Validate props and separate extra attributes
        validated_props, extra_attrs = self._validate_props(kwargs)

        # Call the compiled render function
        try:
            template = self._render(
                **validated_props,
                __children__=children,
                __attrs__=extra_attrs
            )
        except Exception:
            # Let exceptions propagate naturally with proper stack traces
            raise

        # Convert t-string result to Node
        if template is None:
            return Fragment(children=[])
        return tdom_html(template)

    def __str__(self) -> str:
        """Render component with default props when printed."""
        return render(self())

    @property
    def render_code(self) -> str:
        """Get the generated render function source for debugging."""
        return self._render_code

    def _validate_props(self, provided_props: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
        """Validate props and separate extra attributes. Returns (validated_props, extra_attrs)."""
        validated = {}
        extra_attrs = {}

        for name, prop in self._props.items():
            # Check if this is a dependency that should be resolved from context
            if context.is_dependency(prop.type_name):
                # Try to get from context
                ctx_value = context.get(name)
                if ctx_value is not None:
                    validated[name] = ctx_value
                    continue
                # If not in context and not provided, it's required but missing
                if name not in provided_props and not prop.has_default:
                    raise PropValidationError(
                        f"{self.name}.{name}: '{prop.type_name}' dependency not available in request context"
                    )

            if name in provided_props:
                value = provided_props[name]

                # Type validation (only if type was resolved at compile time)
                if value is not None and prop.type_hint:
                    if not isinstance(value, prop.type_hint):
                        actual_type = type(value).__name__
                        expected_type = prop.type_name or prop.type_hint.__name__
                        raise PropValidationError(
                            f"{self.name}.{name}: expected {expected_type}, got {actual_type}",
                            path=self.path,
                            component_name=self.name,
                            props=self._props,
                        )

                # Convert booleans to strings for HTML rendering
                validated[name] = str(value) if isinstance(value, bool) else value
            elif prop.has_default:
                # Use default value (convert bool to string)
                default = prop.default
                validated[name] = str(default) if isinstance(default, bool) else default
            else:
                # Required prop missing
                raise PropValidationError(
                    f"{self.name}: missing required prop '{name}'",
                    path=self.path,
                    component_name=self.name,
                    props=self._props,
                )

        # Collect extra attributes not declared as props
        for name, value in provided_props.items():
            if name not in self._props:
                extra_attrs[name] = value

        return validated, extra_attrs


def _resolve_type(type_name: str, namespace: dict) -> type | None:
    """Resolve type name to actual type from namespace.

    Checks namespace first (imports), then built-ins.
    Returns None if type can't be resolved.
    """
    # Check namespace (imported types, component's scope)
    if type_name in namespace:
        obj = namespace[type_name]
        # Make sure it's actually a type
        if isinstance(obj, type):
            return obj

    # Check built-ins
    try:
        return eval(type_name, {"__builtins__": __builtins__}, {})
    except:
        pass

    return None


def load_component(path: Path | str) -> Component:
    """Load Python file as callable component."""
    import ast
    import importlib.util
    import sys
    from hyper_templates.compiler import ComponentCompiler
    from hyper_templates.errors import ComponentCompileError

    path = Path(path).resolve()
    if not path.exists():
        raise ComponentNotFoundError(f"Component file not found: {path}", path=path)

    # Read source code
    code = path.read_text()

    # Extract props (types not resolved yet)
    props_dict = extract_props(code)

    # Create a version of the code with only imports (no t-string)
    # This prevents NameError when executing the module
    try:
        tree = ast.parse(code)
        import_only_body = [
            node for node in tree.body
            if isinstance(node, (ast.Import, ast.ImportFrom, ast.AnnAssign))
        ]
        import_only_tree = ast.Module(body=import_only_body, type_ignores=[])
        import_only_source = ast.unparse(import_only_tree)
    except SyntaxError:
        # If we can't parse, just use empty code (no imports)
        import_only_source = ""

    # Create a temporary module to resolve imports
    module_name = f"__component_{path.stem}_{id(path)}"
    spec = importlib.util.spec_from_loader(
        module_name,
        loader=None,
        origin=str(path)
    )

    module = importlib.util.module_from_spec(spec)
    module.__file__ = str(path)
    sys.modules[module_name] = module

    # Execute the import-only version to populate module namespace
    if import_only_source:
        try:
            exec(import_only_source, module.__dict__)
        except Exception as e:
            sys.modules.pop(module_name, None)
            raise ComponentCompileError(
                f"Error importing component {path.name}",
                path=path,
                original_error=e
            ) from e

    # Resolve types at compile time
    resolved_props = {}
    for name, prop in props_dict.items():
        if prop.type_name:
            # Check if this is a framework dependency first
            if context.is_dependency(prop.type_name):
                # Don't resolve dependencies - they're injected at runtime
                resolved_props[name] = prop
            else:
                # Try to resolve regular types from namespace
                resolved_type = _resolve_type(prop.type_name, module.__dict__)

                if resolved_type is None:
                    raise ComponentCompileError(
                        f"{path.stem}.{name}: type '{prop.type_name}' not found\n"
                        f"→ Add import: from ... import {prop.type_name}",
                        path=path
                    )

                # Create new Prop with resolved type
                resolved_props[name] = Prop(
                    name=prop.name,
                    type_hint=resolved_type,  # Actual type object
                    default=prop.default,
                    has_default=prop.has_default,
                    type_name=prop.type_name,  # Keep for error messages
                )
        else:
            resolved_props[name] = prop

    # Preprocess code: replace {...} with {__slot__}
    code_processed = re.sub(r'\{\.\.\.(\s*)\}', r'{__slot__\1}', code)

    # Compile the component into a render function
    try:
        compiler = ComponentCompiler(code_processed, path, module.__dict__, resolved_props)
        render_fn, generated_code = compiler.compile()
    except Exception as e:
        raise ComponentCompileError(
            f"Error compiling {path.name}",
            path=path,
            original_error=e
        ) from e
    finally:
        # Clean up sys.modules
        sys.modules.pop(module_name, None)

    return Component(
        path=path,
        name=path.stem,
        _props=resolved_props,  # Types now resolved!
        code=code,
        _render=render_fn,
        _render_code=generated_code,
    )


def render(template: Any) -> str:
    """Convert t-string, Node, or value to HTML string."""
    if isinstance(template, str):
        return template
    if isinstance(template, Node):
        return str(template)
    if hasattr(template, "strings"):
        return str(tdom_html(template))
    return str(template)
