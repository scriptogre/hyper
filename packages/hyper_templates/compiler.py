"""Compile component source into callable render function."""

import ast
import linecache
from pathlib import Path
from typing import Any, Callable

from markupsafe import Markup
from hyper_templates.loader import Prop


class ComponentCompiler:
    """Transforms component source into a render function.

    Compiles component source code into a Python function that can be called
    with different prop values to generate t-strings. This happens once at
    load time, eliminating the need for exec() on every render.
    """

    def __init__(self, source: str, path: Path, module_namespace: dict, props: dict[str, Prop]):
        """Initialize compiler with component source.

        Args:
            source: Component source code (with {...} already replaced by {__slot__})
            path: Path to component file (for debugging)
            module_namespace: Module's namespace (for imports)
            props: Resolved props dict
        """
        self.source = source
        self.path = path
        self.module_namespace = module_namespace
        self.props = props

    def compile(self) -> tuple[Callable, str]:
        """Compile component into a render function.

        Returns:
            Tuple of (render_function, generated_source)
        """
        # Parse source to AST
        try:
            tree = ast.parse(self.source, filename=str(self.path))
        except SyntaxError as e:
            # Re-raise with component context
            raise SyntaxError(
                f"Syntax error in component {self.path.name}",
                (str(self.path), e.lineno, e.offset, e.text)
            ) from e

        # Build the render function AST
        func_def = self._build_render_function(tree)

        # Wrap in a module
        module = ast.Module(body=[func_def], type_ignores=[])
        ast.fix_missing_locations(module)

        # Generate source code for inspection
        generated_source = ast.unparse(module)

        # Compile with real filename for proper stack traces
        code = compile(
            module,
            filename=str(self.path),
            mode="exec"
        )

        # Register source with linecache for debugging
        linecache.cache[str(self.path)] = (
            len(self.source),
            None,
            self.source.splitlines(keepends=True),
            str(self.path)
        )

        # Execute to get the function object
        # Inject module namespace so imports are available
        namespace = {**self.module_namespace, "__builtins__": __builtins__, "Markup": Markup}
        exec(code, namespace)

        _render = namespace["__render__"]

        return _render, generated_source

    def _build_render_function(self, tree: ast.Module) -> ast.FunctionDef:
        """Build AST for the __render__ function."""
        # Build function arguments from props
        args = self._build_arguments()

        # Build function body
        body = self._build_body(tree)

        # Create the function definition
        func = ast.FunctionDef(
            name="__render__",
            args=args,
            body=body,
            decorator_list=[],
            returns=None,
            lineno=1,
            col_offset=0,
        )

        return func

    def _build_arguments(self) -> ast.arguments:
        """Build function arguments from component props."""
        regular_args = []
        defaults = []

        # Convert props to function parameters
        for name, info in self.props.items():
            # Create argument with type annotation if available
            arg = ast.arg(
                arg=name,
                annotation=self._type_to_ast(info.type_hint) if info.type_hint else None,
            )
            regular_args.append(arg)

            # Add default value if prop has one
            if info.has_default:
                defaults.append(self._value_to_ast(info.default))

        # Create keyword-only arguments for __children__ and __attrs__
        kwonly_args = [
            ast.arg(arg="__children__", annotation=None),
            ast.arg(arg="__attrs__", annotation=None),
        ]

        kw_defaults = [
            ast.Tuple(elts=[], ctx=ast.Load()),  # () for children
            ast.Dict(keys=[], values=[]),        # {} for attrs
        ]

        return ast.arguments(
            posonlyargs=[],
            args=regular_args,
            vararg=None,
            kwonlyargs=kwonly_args,
            kw_defaults=kw_defaults,
            kwarg=None,
            defaults=defaults,
        )

    def _build_body(self, tree: ast.Module) -> list[ast.stmt]:
        """Transform component body into function body."""
        body = []

        # Add __slot__ computation at the start
        # __slot__ = Markup("".join(str(c) for c in __children__))
        slot_stmt = ast.parse(
            '__slot__ = Markup("".join(str(c) for c in __children__))'
        ).body[0]
        body.append(slot_stmt)

        # Process original statements
        found_tstring = False

        for node in tree.body:
            # Skip prop definitions (annotated assignments)
            if isinstance(node, ast.AnnAssign):
                if isinstance(node.target, ast.Name):
                    if node.target.id in self.props:
                        continue

            # Skip imports (they're already in module namespace)
            if isinstance(node, (ast.Import, ast.ImportFrom)):
                continue

            # Check if this is a bare expression (potentially the t-string)
            if isinstance(node, ast.Expr) and not found_tstring:
                # Transform to return statement
                return_stmt = ast.Return(value=node.value)
                ast.copy_location(return_stmt, node)
                body.append(return_stmt)
                found_tstring = True
                break  # Only process first t-string
            else:
                # Keep other statements (assignments, conditionals, etc.)
                body.append(node)

        # If no t-string found, return None
        if not found_tstring:
            body.append(ast.Return(value=ast.Constant(value=None)))

        return body

    def _type_to_ast(self, type_hint: type | None) -> ast.expr | None:
        """Convert a type to an AST node."""
        if type_hint is None:
            return None

        # Map types to AST names
        type_map = {
            str: "str",
            int: "int",
            float: "float",
            bool: "bool",
            list: "list",
            dict: "dict",
            set: "set",
            tuple: "tuple",
        }

        type_name = type_map.get(type_hint)
        if type_name:
            return ast.Name(id=type_name, ctx=ast.Load())

        return None

    def _value_to_ast(self, value: Any) -> ast.expr:
        """Convert a Python value to an AST node."""
        if value is None:
            return ast.Constant(value=None)
        elif isinstance(value, (str, int, float, bool)):
            return ast.Constant(value=value)
        elif isinstance(value, list):
            return ast.List(
                elts=[self._value_to_ast(item) for item in value],
                ctx=ast.Load()
            )
        elif isinstance(value, dict):
            return ast.Dict(
                keys=[self._value_to_ast(k) for k in value.keys()],
                values=[self._value_to_ast(v) for v in value.values()]
            )
        elif isinstance(value, tuple):
            return ast.Tuple(
                elts=[self._value_to_ast(item) for item in value],
                ctx=ast.Load()
            )
        elif isinstance(value, set):
            return ast.Set(
                elts=[self._value_to_ast(item) for item in value]
            )
        else:
            # Fallback: use constant
            return ast.Constant(value=value)
