"""Prop extraction from Python source via AST parsing."""

import ast
import re
from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class Prop:
    """Component prop metadata."""

    name: str
    type_hint: type | None  # Resolved type (set at compile time)
    default: Any = field(compare=False, hash=False)
    has_default: bool
    type_name: str | None = None  # Type name string (for error messages)

    @classmethod
    def required(cls, name: str, type_hint: type | None = None, type_name: str | None = None) -> "Prop":
        return cls(name, type_hint, None, False, type_name)

    @classmethod
    def with_default(cls, name: str, default: Any, type_hint: type | None = None, type_name: str | None = None) -> "Prop":
        return cls(name, type_hint, default, True, type_name)


def _get_type_name(annotation: ast.expr) -> str | None:
    """Extract type name from AST annotation node.

    Returns: type_name_str or None
    """
    if isinstance(annotation, ast.Name):
        return annotation.id
    return None


def extract_props(source: str) -> dict[str, Prop]:
    """Parse annotated assignments into Prop dict."""
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return {}

    props = {}
    for node in ast.iter_child_nodes(tree):
        if not (isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name)):
            continue

        name = node.target.id
        if name.startswith("_"):
            continue

        type_name = _get_type_name(node.annotation)

        if node.value is not None:
            # Try to evaluate the default value
            try:
                default = ast.literal_eval(node.value)
                props[name] = Prop.with_default(name, default, None, type_name)
            except (ValueError, TypeError, SyntaxError):
                # Can't evaluate default (e.g., function call) - treat as required
                props[name] = Prop.required(name, None, type_name)
        else:
            props[name] = Prop.required(name, None, type_name)

    return props


def extract_template_string(source: str) -> str | None:
    """Find first t-string in source."""
    for pattern in (r't""".*?"""', r"t'''.*?'''", r't".*?"', r"t'.*?'"):
        if match := re.search(pattern, source, re.DOTALL):
            return match.group(0)
    return None
