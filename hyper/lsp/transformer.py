"""
Transform .hyper files into valid Python for Pyright analysis.

The transformer:
1. Preserves props as type-annotated variables
2. Converts control flow (if/for/match with 'end') to proper Python
3. Converts HTML elements to placeholder expressions
4. Builds a source map for position translation

Example:
    Input (.hyper):
        user: User
        show_badge: bool = True

        <div class="card">
            if user.is_admin:
                <span>Admin</span>
            end
        </div>

    Output (.py):
        user: User
        show_badge: bool = True

        __h__("<div>", class_="card")
        if user.is_admin:
            __h__("<span>", "Admin", "</span>")
        __h__("</div>")
"""

import re
from dataclasses import dataclass, field
from typing import Optional

from .sourcemap import SourceMap, Position, Range


@dataclass
class TransformResult:
    """Result of transforming a .hyper file."""
    python_code: str
    source_map: SourceMap


@dataclass
class TransformContext:
    """Context for tracking state during transformation."""
    output_lines: list[str] = field(default_factory=list)
    source_map: SourceMap = field(default_factory=lambda: SourceMap("", ""))
    current_hyper_line: int = 0
    current_python_line: int = 0
    indent_stack: list[int] = field(default_factory=list)  # Track indentation levels


# Regex patterns
PROP_PATTERN = re.compile(r'^([a-z_][a-z_0-9]*)\s*:\s*(.+?)(?:\s*=\s*(.+))?$')
IF_PATTERN = re.compile(r'^(\s*)(if)\s+(.+):$')
ELIF_PATTERN = re.compile(r'^(\s*)(elif)\s+(.+):$')
ELSE_PATTERN = re.compile(r'^(\s*)(else):$')
FOR_PATTERN = re.compile(r'^(\s*)(for)\s+(\w+)\s+(in)\s+(.+):$')
MATCH_PATTERN = re.compile(r'^(\s*)(match)\s+(.+):$')
CASE_PATTERN = re.compile(r'^(\s*)(case)\s+(.+):$')
END_PATTERN = re.compile(r'^(\s*)(end)\s*$')
HTML_TAG_PATTERN = re.compile(r'^(\s*)<([a-zA-Z][a-zA-Z0-9-]*)')
EXPRESSION_PATTERN = re.compile(r'\{([^}]+)\}')


def transform(hyper_content: str, hyper_uri: str) -> TransformResult:
    """
    Transform .hyper content into valid Python.

    Args:
        hyper_content: The content of the .hyper file
        hyper_uri: The URI of the .hyper file (for source map)

    Returns:
        TransformResult with Python code and source map
    """
    python_uri = hyper_uri.replace('.hyper', '_hyper.py')
    ctx = TransformContext(source_map=SourceMap(hyper_uri, python_uri))

    lines = hyper_content.splitlines()

    # Add imports and helper function
    _emit(ctx, "# Generated from .hyper file - do not edit")
    _emit(ctx, "from typing import Any")
    _emit(ctx, "def __h__(*args, **kwargs) -> Any: ...")  # HTML placeholder
    _emit(ctx, "")

    in_props = True  # Start assuming we're in props section

    for hyper_line_num, line in enumerate(lines):
        ctx.current_hyper_line = hyper_line_num

        # Empty line
        if not line.strip():
            _emit_with_mapping(ctx, "")
            continue

        # Check if we've left props section (first HTML or control flow)
        if in_props:
            if line.strip().startswith('<') or _is_control_flow(line):
                in_props = False
                _emit(ctx, "")  # Add separator after props

        # Props
        if in_props:
            prop_match = PROP_PATTERN.match(line.strip())
            if prop_match:
                name, type_hint, default = prop_match.groups()
                if default:
                    _emit_with_mapping(ctx, f"{name}: {type_hint} = {default}")
                else:
                    _emit_with_mapping(ctx, f"{name}: {type_hint}")
                continue

        # Control flow: if
        if_match = IF_PATTERN.match(line)
        if if_match:
            indent, _, condition = if_match.groups()
            _emit_with_mapping(ctx, f"{indent}if {condition}:")
            continue

        # Control flow: elif
        elif_match = ELIF_PATTERN.match(line)
        if elif_match:
            indent, _, condition = elif_match.groups()
            _emit_with_mapping(ctx, f"{indent}elif {condition}:")
            continue

        # Control flow: else
        else_match = ELSE_PATTERN.match(line)
        if else_match:
            indent, _ = else_match.groups()
            _emit_with_mapping(ctx, f"{indent}else:")
            continue

        # Control flow: for
        for_match = FOR_PATTERN.match(line)
        if for_match:
            indent, _, var, _, iterable = for_match.groups()
            _emit_with_mapping(ctx, f"{indent}for {var} in {iterable}:")
            continue

        # Control flow: match
        match_match = MATCH_PATTERN.match(line)
        if match_match:
            indent, _, subject = match_match.groups()
            _emit_with_mapping(ctx, f"{indent}match {subject}:")
            continue

        # Control flow: case
        case_match = CASE_PATTERN.match(line)
        if case_match:
            indent, _, pattern = case_match.groups()
            _emit_with_mapping(ctx, f"{indent}case {pattern}:")
            continue

        # Control flow: end
        end_match = END_PATTERN.match(line)
        if end_match:
            indent = end_match.group(1)
            # 'end' becomes 'pass' if the previous line was a control statement
            # or we just skip it and Python's indentation handles the rest
            _emit_with_mapping(ctx, f"{indent}pass  # end")
            continue

        # HTML element
        html_match = HTML_TAG_PATTERN.match(line)
        if html_match:
            indent = html_match.group(1)
            # Convert to __h__ call, extracting any expressions
            py_line = _transform_html_line(line, indent)
            _emit_with_mapping(ctx, py_line)
            continue

        # Expression line (just {something})
        if line.strip().startswith('{') and line.strip().endswith('}'):
            indent = len(line) - len(line.lstrip())
            expr = line.strip()[1:-1]
            _emit_with_mapping(ctx, " " * indent + f"__h__({expr})")
            continue

        # Fallback: emit as comment
        _emit_with_mapping(ctx, f"# {line}")

    return TransformResult(
        python_code='\n'.join(ctx.output_lines),
        source_map=ctx.source_map
    )


def _is_control_flow(line: str) -> bool:
    """Check if a line is a control flow statement."""
    stripped = line.strip()
    return any([
        IF_PATTERN.match(line),
        ELIF_PATTERN.match(line),
        ELSE_PATTERN.match(line),
        FOR_PATTERN.match(line),
        MATCH_PATTERN.match(line),
        CASE_PATTERN.match(line),
        END_PATTERN.match(line),
    ])


def _emit(ctx: TransformContext, line: str) -> None:
    """Emit a line without mapping."""
    ctx.output_lines.append(line)
    ctx.current_python_line += 1


def _emit_with_mapping(ctx: TransformContext, line: str) -> None:
    """Emit a line and add source map entry."""
    ctx.output_lines.append(line)
    ctx.source_map.add_line_mapping(ctx.current_hyper_line, ctx.current_python_line)
    ctx.current_python_line += 1


def _transform_html_line(line: str, indent: str) -> str:
    """
    Transform an HTML line to a Python __h__() call.

    Extracts expressions from {brackets} and makes them Python arguments.
    """
    # Simple approach: wrap the whole line in __h__()
    # Extract expressions for type checking
    expressions = EXPRESSION_PATTERN.findall(line)
    if expressions:
        # Create a call that references the expressions
        expr_str = ', '.join(expressions)
        return f"{indent}__h__({expr_str})  # {line.strip()}"
    else:
        return f"{indent}__h__()  # {line.strip()}"
