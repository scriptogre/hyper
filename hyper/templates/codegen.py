"""Code generator for templates.

Walks TNode tree and emits Python source code.
The generated code uses f-strings and runtime utilities for efficient rendering.
"""

from dataclasses import dataclass, field
from string.templatelib import Template, Interpolation

from .loader import Prop
from ._tdom.nodes import (
    TNode,
    TElement,
    TFragment,
    TText,
    TComment,
    TDocumentType,
    TComponent,
    TConditional,
    TMatch,
    TAttribute,
    StaticAttribute,
    InterpolatedAttribute,
    TemplatedAttribute,
    SpreadAttribute,
    VOID_ELEMENTS,
)


def escape_string_for_python(s: str, quote: str = '"') -> str:
    """Escape a string for use in a Python string literal.

    Args:
        s: The string to escape
        quote: The quote character being used (' or ")

    Returns:
        The escaped string (without surrounding quotes)
    """
    # Escape backslashes first
    s = s.replace("\\", "\\\\")
    # Escape the quote character
    s = s.replace(quote, "\\" + quote)
    # Escape newlines and other special characters
    s = s.replace("\n", "\\n")
    s = s.replace("\r", "\\r")
    s = s.replace("\t", "\\t")
    return s


def escape_for_fstring(s: str) -> str:
    """Escape a string for use inside an f-string.

    Args:
        s: The string to escape

    Returns:
        The escaped string with braces doubled
    """
    s = escape_string_for_python(s, quote='"')
    # Double braces for f-string
    s = s.replace("{", "{{")
    s = s.replace("}", "}}")
    return s


@dataclass
class CodeGenContext:
    """Context for code generation."""

    interpolations: tuple[Interpolation, ...]
    props: dict[str, Prop]
    indent: int = 0
    _temp_counter: int = field(default=0, repr=False)

    def get_temp_var(self, prefix: str = "_t") -> str:
        """Generate a unique temporary variable name."""
        self._temp_counter += 1
        return f"{prefix}{self._temp_counter}"

    def get_expression(self, interpolation_index: int) -> str:
        """Get the Python expression for an interpolation."""
        ip = self.interpolations[interpolation_index]
        # Use expression if available, otherwise use repr of value
        if ip.expression:
            return ip.expression
        return repr(ip.value)

    def indent_str(self) -> str:
        """Get the current indentation string."""
        return "    " * self.indent

    def indented(self, delta: int = 1) -> "CodeGenContext":
        """Return a new context with increased indentation."""
        return CodeGenContext(
            interpolations=self.interpolations,
            props=self.props,
            indent=self.indent + delta,
            _temp_counter=self._temp_counter,
        )


class CodeGenerator:
    """Generates Python code from TNode tree."""

    def __init__(
        self, template: Template, props: dict[str, Prop], pre_template_stmts: list
    ):
        self.template = template
        self.props = props
        self.pre_template_stmts = pre_template_stmts

    def generate(self, tree: TNode) -> str:
        """Generate Python source code from the parsed tree.

        Returns the complete Python module source.
        """
        ctx = CodeGenContext(
            interpolations=self.template.interpolations,
            props=self.props,
        )

        # Generate the render function body
        render_body = self._generate_node(tree, ctx)

        # Build the module
        lines = [
            "from hyper.templates.runtime import (",
            "    escape_html,",
            "    format_classes,",
            "    format_styles,",
            "    format_attrs,",
            "    render_data_attrs,",
            "    render_aria_attrs,",
            ")",
            "from markupsafe import Markup",
            "",
            "",
        ]

        # Generate render function signature
        params = self._generate_params()
        lines.append(f"def render({params}) -> str:")

        # Add __slot__ computation at the start (for children)
        lines.append('    __slot__ = Markup("".join(str(c) for c in __children__))')
        lines.append("")

        # Add pre-template statements (arbitrary Python code)
        if self.pre_template_stmts:
            import ast

            for stmt in self.pre_template_stmts:
                stmt_code = ast.unparse(stmt)
                # Add proper indentation for each line
                for line in stmt_code.split("\n"):
                    lines.append(f"    {line}")
            lines.append("")

        # Add the body
        if isinstance(render_body, str) and render_body.startswith("return "):
            # Simple case: just a return expression
            lines.append(f"    {render_body}")
        else:
            # Complex case: multiple statements
            for line in render_body.split("\n"):
                if line.strip():
                    lines.append(f"    {line}")

        lines.append("")
        lines.append("")
        lines.append("# Public API")
        lines.append("__call__ = render")

        return "\n".join(lines)

    def _generate_params(self) -> str:
        """Generate function parameter list from props."""
        params = []
        for name, prop in self.props.items():
            if prop.type_name:
                param = f"{name}: {prop.type_name}"
            else:
                param = name

            if prop.has_default:
                default = repr(prop.default)
                param = f"{param} = {default}"

            params.append(param)

        # Add special parameters for children and extra attrs
        params.append("__children__: tuple = ()")
        params.append("__attrs__: dict = {}")

        return ", ".join(params)

    def _generate_node(self, node: TNode, ctx: CodeGenContext) -> str:
        """Generate code for a node, dispatching by type."""
        match node:
            case TElement():
                return self._generate_element(node, ctx)
            case TFragment():
                return self._generate_fragment(node, ctx)
            case TText():
                return self._generate_text(node, ctx)
            case TComment():
                return self._generate_comment(node, ctx)
            case TDocumentType():
                return self._generate_doctype(node, ctx)
            case TComponent():
                return self._generate_component(node, ctx)
            case TConditional():
                return self._generate_conditional(node, ctx)
            case TMatch():
                return self._generate_match(node, ctx)
            case _:
                raise ValueError(f"Unknown node type: {type(node)}")

    def _generate_element(self, node: TElement, ctx: CodeGenContext) -> str:
        """Generate code for an element node."""
        tag = node.tag

        # Generate attributes
        attrs_code = self._generate_attrs(node.attrs, ctx)

        # Handle void elements (no children)
        if tag in VOID_ELEMENTS:
            if attrs_code:
                return f'return "<{tag}" + {attrs_code} + " />"'
            else:
                return f'return "<{tag} />"'

        # Generate children
        children_code = self._generate_children(node.children, ctx)

        # Check if children_code has pre-statements (multi-line with return)
        if "\nreturn " in children_code:
            # Split into pre-statements and the result expression
            lines = children_code.split("\n")
            pre_stmts = []
            result_expr = "''"
            for i, line in enumerate(lines):
                if line.strip().startswith("return "):
                    result_expr = line.strip()[7:]
                    pre_stmts = lines[:i]
                    break

            # Build the element with pre-statements
            pre_code = "\n".join(pre_stmts)
            if attrs_code:
                return f'{pre_code}\nreturn "<{tag}" + {attrs_code} + ">" + {result_expr} + "</{tag}>"'
            else:
                return f'{pre_code}\nreturn "<{tag}>" + {result_expr} + "</{tag}>"'

        # Simple case: no pre-statements
        if attrs_code and children_code:
            return (
                f'return "<{tag}" + {attrs_code} + ">" + {children_code} + "</{tag}>"'
            )
        elif attrs_code:
            return f'return "<{tag}" + {attrs_code} + "></{tag}>"'
        elif children_code:
            return f'return "<{tag}>" + {children_code} + "</{tag}>"'
        else:
            return f'return "<{tag}></{tag}>"'

    def _generate_fragment(self, node: TFragment, ctx: CodeGenContext) -> str:
        """Generate code for a fragment node."""
        if not node.children:
            return "return ''"

        children_code = self._generate_children(node.children, ctx)
        if not children_code:
            return "return ''"

        # If children_code already contains return (multi-line with pre-statements),
        # return it as-is; otherwise wrap in return
        if "\nreturn " in children_code or children_code.startswith("return "):
            return children_code
        return f"return {children_code}"

    def _generate_text(self, node: TText, ctx: CodeGenContext) -> str:
        """Generate code for a text node."""
        parts = list(node.text_t)

        if not parts:
            return 'return ""'

        if len(parts) == 1 and isinstance(parts[0], str):
            # Pure static text
            escaped = escape_string_for_python(parts[0])
            return f'return "{escaped}"'

        # Mixed content - build f-string parts
        result_parts = []
        for part in parts:
            if isinstance(part, str):
                # Static text - escape for f-string
                escaped = escape_for_fstring(part)
                result_parts.append(escaped)
            else:
                # Interpolation
                expr = ctx.get_expression(part.value)
                result_parts.append(f"{{escape_html({expr})}}")

        return f'return f"{"".join(result_parts)}"'

    def _generate_comment(self, node: TComment, ctx: CodeGenContext) -> str:
        """Generate code for a comment node."""
        parts = list(node.text_t)

        if len(parts) == 1 and isinstance(parts[0], str):
            text = escape_string_for_python(parts[0])
            return f'return "<!--{text}-->"'

        result_parts = []
        for part in parts:
            if isinstance(part, str):
                escaped = escape_for_fstring(part)
                result_parts.append(escaped)
            else:
                expr = ctx.get_expression(part.value)
                result_parts.append(f"{{escape_html({expr})}}")

        return f'return f"<!--{"".join(result_parts)}-->"'

    def _generate_doctype(self, node: TDocumentType, ctx: CodeGenContext) -> str:
        """Generate code for a doctype node."""
        return f"return '<!DOCTYPE {node.text}>'"

    def _generate_component(self, node: TComponent, ctx: CodeGenContext) -> str:
        """Generate code for a component invocation."""
        # Get the component expression
        comp_expr = ctx.get_expression(node.starttag_interpolation_index)

        # Generate children
        if node.children:
            children_parts = []
            for child in node.children:
                child_code = self._generate_node(child, ctx)
                # Extract the expression from "return X"
                if child_code.startswith("return "):
                    children_parts.append(child_code[7:])
            children_expr = " + ".join(children_parts) if children_parts else "''"
        else:
            children_expr = "()"

        # Generate attributes as kwargs
        kwargs = self._generate_component_kwargs(node.attrs, ctx)

        # Build the component call
        if kwargs and children_expr != "()":
            return f"return str({comp_expr}(children=({children_expr},), {kwargs}))"
        elif kwargs:
            return f"return str({comp_expr}({kwargs}))"
        elif children_expr != "()":
            return f"return str({comp_expr}(children=({children_expr},)))"
        else:
            return f"return str({comp_expr}())"

    def _generate_conditional(self, node: TConditional, ctx: CodeGenContext) -> str:
        """Generate code for if/elif/else conditional."""
        lines = []
        var = ctx.get_temp_var("_cond")

        for i, branch in enumerate(node.branches):
            children_code = self._generate_children(branch.children, ctx)

            if branch.condition_index is None:
                # else branch
                lines.append("else:")
                lines.append(f"    {var} = {children_code}")
            elif i == 0:
                # if branch
                cond_expr = ctx.get_expression(branch.condition_index)
                lines.append(f"if {cond_expr}:")
                lines.append(f"    {var} = {children_code}")
            else:
                # elif branch
                cond_expr = ctx.get_expression(branch.condition_index)
                lines.append(f"elif {cond_expr}:")
                lines.append(f"    {var} = {children_code}")

        # Handle case where no else branch (default to empty string)
        if node.branches[-1].condition_index is not None:
            lines.append("else:")
            lines.append(f"    {var} = ''")

        lines.append(f"return {var}")
        return "\n".join(lines)

    def _generate_match(self, node: TMatch, ctx: CodeGenContext) -> str:
        """Generate code for match/case pattern matching."""
        subject_expr = ctx.get_expression(node.subject_index)
        var = ctx.get_temp_var("_match")
        lines = [f"match {subject_expr}:"]

        for case in node.cases:
            pattern_expr = ctx.get_expression(case.pattern_index)
            children_code = self._generate_children(case.children, ctx)
            lines.append(f"    case {pattern_expr}:")
            lines.append(f"        {var} = {children_code}")

        # Default case
        lines.append("    case _:")
        lines.append(f"        {var} = ''")

        lines.append(f"return {var}")
        return "\n".join(lines)

    def _generate_children(
        self, children: tuple[TNode, ...], ctx: CodeGenContext
    ) -> str:
        """Generate code to render children and concatenate them."""
        if not children:
            return "''"

        parts = []
        pre_statements = []

        for child in children:
            child_code = self._generate_node(child, ctx)

            # Single-line: extract expression from "return X"
            if child_code.startswith("return "):
                parts.append(child_code[7:])
            elif "\n" in child_code:
                # Multi-line code (conditionals, match) - extract statements and return var
                lines = child_code.split("\n")
                last_line = lines[-1].strip()
                if last_line.startswith("return "):
                    # Add all lines except the last as pre-statements
                    pre_statements.extend(lines[:-1])
                    # Extract the variable from the return
                    parts.append(last_line[7:])
                else:
                    raise ValueError(f"Multi-line code must end with return: {child_code}")
            else:
                raise ValueError(f"Unexpected code format: {child_code}")

        if len(parts) == 1 and not pre_statements:
            return parts[0]

        # If we have pre-statements, we need to return them along with the expression
        # This is handled by the caller who will see the newlines
        result_expr = " + ".join(parts)
        if pre_statements:
            return "\n".join(pre_statements) + f"\nreturn {result_expr}"

        return result_expr

    def _generate_attrs(
        self, attrs: tuple[TAttribute, ...], ctx: CodeGenContext
    ) -> str:
        """Generate code for element attributes."""
        if not attrs:
            return ""

        parts = []
        for attr in attrs:
            attr_code = self._generate_attr(attr, ctx)
            if attr_code:
                parts.append(attr_code)

        if not parts:
            return ""

        return " + ".join(parts)

    def _generate_attr(self, attr: TAttribute, ctx: CodeGenContext) -> str:
        """Generate code for a single attribute."""
        match attr:
            case StaticAttribute(name=name, value=value):
                if value is None:
                    # Boolean attribute
                    return f'" {name}"'
                else:
                    escaped = escape_string_for_python(value)
                    return f'" {name}=\\"{escaped}\\""'

            case InterpolatedAttribute(name=name, interpolation_index=idx):
                expr = ctx.get_expression(idx)
                # Handle special attributes
                if name == "class":
                    return f'" class=\\"" + format_classes({expr}) + "\\""'
                elif name == "style":
                    return f'" style=\\"" + format_styles({expr}) + "\\""'
                elif name == "data":
                    return f"render_data_attrs({expr})"
                elif name == "aria":
                    return f"render_aria_attrs({expr})"
                else:
                    # Standard attribute - handle True/False/None
                    # Use conditional expression for inline evaluation
                    return f'("" if ({expr}) is False or ({expr}) is None else (" {name}" if ({expr}) is True else " {name}=\\"" + str(escape_html({expr})) + "\\""))'

            case TemplatedAttribute(name=name, value_t=value_t):
                # Mix of static and dynamic parts - build concatenation
                parts = []
                for part in value_t:
                    if isinstance(part, str):
                        escaped = escape_string_for_python(part)
                        parts.append(f'"{escaped}"')
                    else:
                        expr = ctx.get_expression(part.value)
                        parts.append(f"str(escape_html({expr}))")
                value_expr = " + ".join(parts) if parts else '""'
                return f'" {name}=\\"" + {value_expr} + "\\""'

            case SpreadAttribute(interpolation_index=idx):
                expr = ctx.get_expression(idx)
                return f"format_attrs({expr})"

        return ""

    def _generate_component_kwargs(
        self, attrs: tuple[TAttribute, ...], ctx: CodeGenContext
    ) -> str:
        """Generate keyword arguments for a component call."""
        kwargs = []
        for attr in attrs:
            match attr:
                case StaticAttribute(name=name, value=value):
                    # Convert kebab-case to snake_case
                    param_name = name.replace("-", "_")
                    if value is None:
                        kwargs.append(f"{param_name}=True")
                    else:
                        kwargs.append(f"{param_name}={repr(value)}")

                case InterpolatedAttribute(name=name, interpolation_index=idx):
                    param_name = name.replace("-", "_")
                    expr = ctx.get_expression(idx)
                    kwargs.append(f"{param_name}={expr}")

                case TemplatedAttribute(name=name, value_t=value_t):
                    param_name = name.replace("-", "_")
                    # Build the templated value
                    parts = []
                    for part in value_t:
                        if isinstance(part, str):
                            parts.append(part.replace("{", "{{").replace("}", "}}"))
                        else:
                            expr = ctx.get_expression(part.value)
                            parts.append(f"{{{expr}}}")
                    kwargs.append(f"{param_name}=f'{''.join(parts)}'")

                case SpreadAttribute(interpolation_index=idx):
                    expr = ctx.get_expression(idx)
                    kwargs.append(f"**{expr}")

        return ", ".join(kwargs)


def generate_code(
    template: Template,
    tree: TNode,
    props: dict[str, Prop],
    pre_template_stmts: list | None = None,
) -> str:
    """Generate Python source code from a parsed template.

    Args:
        template: The parsed Template object (for interpolation expressions)
        tree: The TNode tree from parsing
        props: Props dict for function signature
        pre_template_stmts: Optional list of AST statements to include before return

    Returns:
        Complete Python module source code
    """
    generator = CodeGenerator(template, props, pre_template_stmts or [])
    return generator.generate(tree)
