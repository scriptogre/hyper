"""Compile template source into callable render function.

This module transforms template source code into Python functions.
The generated code uses f-strings and runtime utilities for efficient rendering.
"""

import ast
import linecache
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

from markupsafe import Markup

from hyper.templates.loader import Prop
from hyper.templates._tdom.parser import parse_html
from hyper.templates.codegen import generate_code


# Sentinel for missing values
MISSING = object()


@dataclass
class CompiledTemplate:
    """Result of template compilation."""

    source: str  # Generated Python source
    render_func: Callable  # Main render function
    metadata: "TemplateMetadata"


@dataclass
class TemplateMetadata:
    """Metadata about a compiled template."""

    props: dict[str, Prop]
    is_async: bool = False
    has_streaming: bool = False


def is_debug_mode() -> bool:
    """Check if debug mode is enabled via environment variable."""
    debug = os.environ.get("DEBUG", "").lower()
    return debug in ("1", "true", "yes", "on")


def write_debug_file(path: Path, source: str) -> None:
    """Write generated code to a .gen.py file for debugging."""
    gen_path = path.with_suffix(".gen.py")
    gen_path.write_text(source)


@dataclass
class MockInterpolation:
    """Mock interpolation that provides expression info for codegen."""

    value: Any
    expression: str
    conversion: int = -1
    format_spec: str = ""


@dataclass
class MockTemplate:
    """Mock template that provides structure info for parsing."""

    strings: tuple[str, ...]
    interpolations: tuple[MockInterpolation, ...]


class TemplateCompiler:
    """Compiles template source into a render function.

    Takes a template Python file, extracts props and t-string,
    parses the HTML, and generates Python code that efficiently
    renders the template.
    """

    def __init__(
        self,
        source: str,
        path: Path,
        module_namespace: dict,
        props: dict[str, Prop],
    ):
        """Initialize compiler with template source.

        Args:
            source: Template source code (with {...} already replaced by {__slot__})
            path: Path to template file (for debugging)
            module_namespace: Module's namespace (for imports)
            props: Resolved props dict
        """
        self.source = source
        self.path = path
        self.module_namespace = module_namespace
        self.props = props

    def compile(self) -> tuple[Callable, str]:
        """Compile template into a render function.

        Returns:
            Tuple of (render_function, generated_source)
        """
        # Parse source and extract template structure + pre-template code
        template, pre_template_stmts = self._extract_template_and_stmts()

        if template is None:
            # No t-string found - return a function that returns empty string
            def empty_render(**kwargs):
                return Markup("")

            return empty_render, "def render(**kwargs): return Markup('')"

        # Parse the HTML template into a TNode tree
        tree = parse_html(template)

        # Generate Python code from the tree
        generated_source = generate_code(template, tree, self.props, pre_template_stmts)

        # Write debug file if in debug mode
        if is_debug_mode():
            write_debug_file(self.path, generated_source)

        # Compile the generated source
        code = compile(generated_source, filename=str(self.path), mode="exec")

        # Register source with linecache for debugging
        linecache.cache[str(self.path)] = (
            len(generated_source),
            None,
            generated_source.splitlines(keepends=True),
            str(self.path),
        )

        # Execute to get the render function
        namespace = {
            **self.module_namespace,
            "__builtins__": __builtins__,
            "Markup": Markup,
        }
        exec(code, namespace)  # nosec B102 - executing compiled template code

        render_func = namespace["render"]

        return render_func, generated_source

    def _extract_template_and_stmts(
        self,
    ) -> tuple[MockTemplate | None, list[ast.stmt]]:
        """Extract template structure and pre-template statements from AST.

        Returns:
            Tuple of (mock_template, pre_template_statements)
        """
        try:
            tree = ast.parse(self.source, filename=str(self.path))
        except SyntaxError as e:
            raise SyntaxError(
                f"Syntax error in template {self.path.name}",
                (str(self.path), e.lineno, e.offset, e.text),
            ) from e

        pre_template_stmts = []
        template_node = None

        for node in tree.body:
            if isinstance(node, ast.Expr) and isinstance(node.value, ast.TemplateStr):
                # Found the t-string
                template_node = node.value
                break
            elif isinstance(node, ast.AnnAssign):
                # Skip prop definitions (type annotations)
                if isinstance(node.target, ast.Name):
                    if node.target.id in self.props:
                        continue
                # Other annotated assignments are included
                pre_template_stmts.append(node)
            elif isinstance(node, (ast.Import, ast.ImportFrom)):
                # Skip imports (they're in module namespace)
                continue
            else:
                # Include other statements (assignments, if, etc.)
                pre_template_stmts.append(node)

        if template_node is None:
            return None, []

        # Build MockTemplate from AST
        template = self._build_mock_template(template_node)

        return template, pre_template_stmts

    def _build_mock_template(self, node: ast.TemplateStr) -> MockTemplate:
        """Build a MockTemplate from an AST TemplateStr node.

        This extracts the structure without evaluating any expressions.
        """
        strings = []
        interpolations = []

        current_string = ""
        for value in node.values:
            if isinstance(value, ast.Constant):
                current_string += str(value.value)
            elif isinstance(value, ast.Interpolation):
                # End the current string
                strings.append(current_string)
                current_string = ""

                # Create mock interpolation with expression text
                expr_text = value.str if value.str else ast.unparse(value.value)
                mock_ip = MockInterpolation(
                    value=len(interpolations),  # Use index as value
                    expression=expr_text,
                    conversion=value.conversion,
                    format_spec=(
                        ast.unparse(value.format_spec) if value.format_spec else ""
                    ),
                )
                interpolations.append(mock_ip)

        # Add final string
        strings.append(current_string)

        return MockTemplate(
            strings=tuple(strings),
            interpolations=tuple(interpolations),
        )


def compile_template(
    source: str,
    path: Path,
    module_namespace: dict | None = None,
    props: dict[str, Prop] | None = None,
) -> CompiledTemplate:
    """Compile a template source file.

    Args:
        source: Template source code
        path: Path to template file
        module_namespace: Optional module namespace for imports
        props: Optional props dict

    Returns:
        CompiledTemplate with render function and metadata
    """
    module_namespace = module_namespace or {}
    props = props or {}

    compiler = TemplateCompiler(source, path, module_namespace, props)
    render_func, generated_source = compiler.compile()

    metadata = TemplateMetadata(
        props=props,
        is_async=False,  # TODO: detect async templates
    )

    return CompiledTemplate(
        source=generated_source,
        render_func=render_func,
        metadata=metadata,
    )
