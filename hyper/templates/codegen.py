"""Code generator for templates.

Generates Python code that builds output by appending to a parts list.
This approach handles control flow (if/match) naturally as regular statements.
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


def escape_string(s: str, quote: str = '"') -> str:
    """Escape a string for use in a Python string literal."""
    s = s.replace("\\", "\\\\")
    s = s.replace(quote, "\\" + quote)
    s = s.replace("\n", "\\n")
    s = s.replace("\r", "\\r")
    s = s.replace("\t", "\\t")
    return s


def escape_for_fstring(s: str) -> str:
    """Escape a string for use inside an f-string."""
    s = escape_string(s, quote='"')
    s = s.replace("{", "{{")
    s = s.replace("}", "}}")
    return s


@dataclass
class CodeGenContext:
    """Context for code generation."""

    interpolations: tuple[Interpolation, ...]
    props: dict[str, Prop]
    indent: int = 1  # Start at 1 (inside function)
    _temp_counter: int = field(default=0, repr=False)

    def get_temp_var(self, prefix: str = "_t") -> str:
        """Generate a unique temporary variable name."""
        self._temp_counter += 1
        return f"{prefix}{self._temp_counter}"

    def get_expression(self, interpolation_index: int) -> str:
        """Get the Python expression for an interpolation."""
        ip = self.interpolations[interpolation_index]
        return ip.expression if ip.expression else repr(ip.value)

    def i(self, line: str = "") -> str:
        """Return line with current indentation."""
        if not line:
            return ""
        return "    " * self.indent + line

    def deeper(self) -> "CodeGenContext":
        """Return context with one more level of indentation."""
        return CodeGenContext(
            interpolations=self.interpolations,
            props=self.props,
            indent=self.indent + 1,
            _temp_counter=self._temp_counter,
        )


class CodeGenerator:
    """Generates Python code from TNode tree using parts-based approach."""

    def __init__(
        self, template: Template, props: dict[str, Prop], pre_template_stmts: list
    ):
        self.template = template
        self.props = props
        self.pre_template_stmts = pre_template_stmts

    def generate(self, tree: TNode) -> str:
        """Generate complete Python module source."""
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

        # Function signature
        params = self._generate_params()
        lines.append(f"def render({params}) -> str:")

        # Initialize
        lines.append('    __slot__ = Markup("".join(str(c) for c in __children__))')
        lines.append("    __p__ = []  # Output parts")
        lines.append("")

        # Pre-template statements
        if self.pre_template_stmts:
            import ast
            for stmt in self.pre_template_stmts:
                for line in ast.unparse(stmt).split("\n"):
                    lines.append(f"    {line}")
            lines.append("")

        # Generate body
        ctx = CodeGenContext(
            interpolations=self.template.interpolations,
            props=self.props,
        )
        body_lines = self._emit(tree, ctx)
        lines.extend(body_lines)

        # Return joined parts
        lines.append("")
        lines.append('    return "".join(__p__)')
        lines.append("")
        lines.append("")
        lines.append("# Public API")
        lines.append("__call__ = render")

        return "\n".join(lines)

    def _generate_params(self) -> str:
        """Generate function parameters from props."""
        params = []
        for name, prop in self.props.items():
            param = f"{name}: {prop.type_name}" if prop.type_name else name
            if prop.has_default:
                param = f"{param} = {repr(prop.default)}"
            params.append(param)

        params.append("__children__: tuple = ()")
        params.append("__attrs__: dict = {}")
        return ", ".join(params)

    def _emit(self, node: TNode, ctx: CodeGenContext) -> list[str]:
        """Emit code lines for a node."""
        match node:
            case TFragment():
                return self._emit_fragment(node, ctx)
            case TElement():
                return self._emit_element(node, ctx)
            case TText():
                return self._emit_text(node, ctx)
            case TComment():
                return self._emit_comment(node, ctx)
            case TDocumentType():
                return self._emit_doctype(node, ctx)
            case TComponent():
                return self._emit_component(node, ctx)
            case TConditional():
                return self._emit_conditional(node, ctx)
            case TMatch():
                return self._emit_match(node, ctx)
            case _:
                raise ValueError(f"Unknown node type: {type(node)}")

    def _emit_fragment(self, node: TFragment, ctx: CodeGenContext) -> list[str]:
        """Emit code for a fragment (just its children)."""
        lines = []
        for child in node.children:
            lines.extend(self._emit(child, ctx))
        return lines

    def _emit_element(self, node: TElement, ctx: CodeGenContext) -> list[str]:
        """Emit code for an element."""
        tag = node.tag
        lines = []

        # Opening tag
        attrs_code = self._attrs_expr(node.attrs, ctx)
        if attrs_code:
            lines.append(ctx.i(f'__p__.append("<{tag}" + {attrs_code} + ">")'))
        else:
            lines.append(ctx.i(f'__p__.append("<{tag}>")'))

        # Handle void elements
        if tag in VOID_ELEMENTS:
            # Replace the line we just added
            if attrs_code:
                lines[-1] = ctx.i(f'__p__.append("<{tag}" + {attrs_code} + " />")')
            else:
                lines[-1] = ctx.i(f'__p__.append("<{tag} />")')
            return lines

        # Children
        for child in node.children:
            lines.extend(self._emit(child, ctx))

        # Closing tag
        lines.append(ctx.i(f'__p__.append("</{tag}>")'))
        return lines

    def _emit_text(self, node: TText, ctx: CodeGenContext) -> list[str]:
        """Emit code for text content."""
        parts = list(node.text_t)
        if not parts:
            return []

        # Pure static text
        if len(parts) == 1 and isinstance(parts[0], str):
            escaped = escape_string(parts[0])
            return [ctx.i(f'__p__.append("{escaped}")')]

        # Mixed content - use f-string
        fstring_parts = []
        for part in parts:
            if isinstance(part, str):
                fstring_parts.append(escape_for_fstring(part))
            else:
                expr = ctx.get_expression(part.value)
                fstring_parts.append(f"{{escape_html({expr})}}")

        return [ctx.i(f'__p__.append(f"{"".join(fstring_parts)}")')]

    def _emit_comment(self, node: TComment, ctx: CodeGenContext) -> list[str]:
        """Emit code for HTML comment."""
        parts = list(node.text_t)

        if len(parts) == 1 and isinstance(parts[0], str):
            escaped = escape_string(parts[0])
            return [ctx.i(f'__p__.append("<!--{escaped}-->")')]

        fstring_parts = []
        for part in parts:
            if isinstance(part, str):
                fstring_parts.append(escape_for_fstring(part))
            else:
                expr = ctx.get_expression(part.value)
                fstring_parts.append(f"{{escape_html({expr})}}")

        return [ctx.i(f'__p__.append(f"<!--{"".join(fstring_parts)}-->")')]

    def _emit_doctype(self, node: TDocumentType, ctx: CodeGenContext) -> list[str]:
        """Emit code for DOCTYPE."""
        return [ctx.i(f'__p__.append("<!DOCTYPE {node.text}>")')]

    def _emit_component(self, node: TComponent, ctx: CodeGenContext) -> list[str]:
        """Emit code for component invocation."""
        comp_expr = ctx.get_expression(node.starttag_interpolation_index)

        # Build children as a tuple of strings
        if node.children:
            # For components, we need to render children to strings
            # Create a temporary parts list for children
            child_var = ctx.get_temp_var("_children")
            lines = [ctx.i(f"{child_var} = []")]

            # Save current __p__ and use child_var
            for child in node.children:
                child_lines = self._emit(child, ctx)
                # Replace __p__ with child_var in the generated lines
                for line in child_lines:
                    lines.append(line.replace("__p__", child_var))

            children_expr = f'"".join({child_var})'
        else:
            lines = []
            children_expr = None

        # Build kwargs
        kwargs = self._component_kwargs(node.attrs, ctx)

        # Build the call
        if kwargs and children_expr:
            call = f'{comp_expr}(children=({children_expr},), {kwargs})'
        elif kwargs:
            call = f'{comp_expr}({kwargs})'
        elif children_expr:
            call = f'{comp_expr}(children=({children_expr},))'
        else:
            call = f'{comp_expr}()'

        lines.append(ctx.i(f'__p__.append(str({call}))'))
        return lines

    def _emit_conditional(self, node: TConditional, ctx: CodeGenContext) -> list[str]:
        """Emit code for if/elif/else."""
        lines = []
        deeper = ctx.deeper()

        for i, branch in enumerate(node.branches):
            if branch.condition_index is None:
                # else
                lines.append(ctx.i("else:"))
            elif i == 0:
                # if
                cond = ctx.get_expression(branch.condition_index)
                lines.append(ctx.i(f"if {cond}:"))
            else:
                # elif
                cond = ctx.get_expression(branch.condition_index)
                lines.append(ctx.i(f"elif {cond}:"))

            # Branch body
            branch_lines = []
            for child in branch.children:
                branch_lines.extend(self._emit(child, deeper))

            if branch_lines:
                lines.extend(branch_lines)
            else:
                lines.append(deeper.i("pass"))

        # If no else, add empty else
        if node.branches[-1].condition_index is not None:
            lines.append(ctx.i("else:"))
            lines.append(deeper.i("pass"))

        return lines

    def _emit_match(self, node: TMatch, ctx: CodeGenContext) -> list[str]:
        """Emit code for match/case."""
        subject = ctx.get_expression(node.subject_index)
        lines = [ctx.i(f"match {subject}:")]
        deeper = ctx.deeper()
        has_wildcard = False

        for case in node.cases:
            pattern = ctx.get_expression(case.pattern_index)
            # Handle {...} wildcard
            if pattern == "__slot__":
                pattern = "_"
                has_wildcard = True

            lines.append(deeper.i(f"case {pattern}:"))

            # Case body
            case_deeper = deeper.deeper()
            case_lines = []
            for child in case.children:
                case_lines.extend(self._emit(child, case_deeper))

            if case_lines:
                lines.extend(case_lines)
            else:
                lines.append(case_deeper.i("pass"))

        # Default case if no wildcard
        if not has_wildcard:
            lines.append(deeper.i("case _:"))
            lines.append(deeper.deeper().i("pass"))

        return lines

    def _attrs_expr(self, attrs: tuple[TAttribute, ...], ctx: CodeGenContext) -> str:
        """Generate expression for element attributes."""
        if not attrs:
            return ""

        parts = []
        for attr in attrs:
            code = self._attr_expr(attr, ctx)
            if code:
                parts.append(code)

        return " + ".join(parts) if parts else ""

    def _attr_expr(self, attr: TAttribute, ctx: CodeGenContext) -> str:
        """Generate expression for a single attribute."""
        match attr:
            case StaticAttribute(name=name, value=value):
                if value is None:
                    return f'" {name}"'
                escaped = escape_string(value)
                return f'" {name}=\\"{escaped}\\""'

            case InterpolatedAttribute(name=name, interpolation_index=idx):
                expr = ctx.get_expression(idx)
                if name == "class":
                    return f'" class=\\"" + format_classes({expr}) + "\\""'
                elif name == "style":
                    return f'" style=\\"" + format_styles({expr}) + "\\""'
                elif name == "data":
                    return f"render_data_attrs({expr})"
                elif name == "aria":
                    return f"render_aria_attrs({expr})"
                else:
                    # Handle True/False/None
                    return (
                        f'("" if ({expr}) is False or ({expr}) is None else '
                        f'(" {name}" if ({expr}) is True else '
                        f'" {name}=\\"" + str(escape_html({expr})) + "\\""))'
                    )

            case TemplatedAttribute(name=name, value_t=value_t):
                parts = []
                for part in value_t:
                    if isinstance(part, str):
                        parts.append(f'"{escape_string(part)}"')
                    else:
                        expr = ctx.get_expression(part.value)
                        parts.append(f"str(escape_html({expr}))")
                value_expr = " + ".join(parts) if parts else '""'
                return f'" {name}=\\"" + {value_expr} + "\\""'

            case SpreadAttribute(interpolation_index=idx):
                expr = ctx.get_expression(idx)
                return f"format_attrs({expr})"

        return ""

    def _component_kwargs(
        self, attrs: tuple[TAttribute, ...], ctx: CodeGenContext
    ) -> str:
        """Generate kwargs for component call."""
        kwargs = []
        for attr in attrs:
            match attr:
                case StaticAttribute(name=name, value=value):
                    param = name.replace("-", "_")
                    if value is None:
                        kwargs.append(f"{param}=True")
                    else:
                        kwargs.append(f"{param}={repr(value)}")

                case InterpolatedAttribute(name=name, interpolation_index=idx):
                    param = name.replace("-", "_")
                    expr = ctx.get_expression(idx)
                    kwargs.append(f"{param}={expr}")

                case TemplatedAttribute(name=name, value_t=value_t):
                    param = name.replace("-", "_")
                    parts = []
                    for part in value_t:
                        if isinstance(part, str):
                            parts.append(part.replace("{", "{{").replace("}", "}}"))
                        else:
                            expr = ctx.get_expression(part.value)
                            parts.append(f"{{{expr}}}")
                    kwargs.append(f"{param}=f'{''.join(parts)}'")

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
    """Generate Python source code from a parsed template."""
    generator = CodeGenerator(template, props, pre_template_stmts or [])
    return generator.generate(tree)
