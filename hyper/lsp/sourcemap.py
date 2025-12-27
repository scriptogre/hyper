"""
Source map for tracking positions between .hyper and generated Python files.

Like how templ maps .templ -> .go positions, we map .hyper -> .py positions.
"""

from dataclasses import dataclass
from typing import Optional


@dataclass
class Position:
    """A position in a file (0-indexed line and character)."""
    line: int
    character: int


@dataclass
class Range:
    """A range in a file."""
    start: Position
    end: Position


@dataclass
class Mapping:
    """Maps a range in the source .hyper file to a range in generated Python."""
    source: Range  # Position in .hyper file
    target: Range  # Position in generated .py file


class SourceMap:
    """
    Bidirectional mapping between .hyper source positions and generated Python positions.

    This enables:
    - Converting editor positions (in .hyper) to Pyright positions (in .py)
    - Converting Pyright responses (in .py) back to editor positions (in .hyper)
    """

    def __init__(self, hyper_uri: str, python_uri: str):
        self.hyper_uri = hyper_uri
        self.python_uri = python_uri
        self.mappings: list[Mapping] = []
        # Line-level mappings for simple cases
        self._hyper_to_python_lines: dict[int, int] = {}
        self._python_to_hyper_lines: dict[int, int] = {}

    def add_line_mapping(self, hyper_line: int, python_line: int) -> None:
        """Add a simple line-to-line mapping."""
        self._hyper_to_python_lines[hyper_line] = python_line
        self._python_to_hyper_lines[python_line] = hyper_line

    def add_mapping(self, source: Range, target: Range) -> None:
        """Add a range mapping."""
        self.mappings.append(Mapping(source=source, target=target))

    def hyper_to_python(self, pos: Position) -> Optional[Position]:
        """Convert a position in .hyper to the corresponding position in generated Python."""
        # First check exact mappings
        for mapping in self.mappings:
            if self._in_range(pos, mapping.source):
                # Calculate offset within source range
                line_offset = pos.line - mapping.source.start.line
                if line_offset == 0:
                    char_offset = pos.character - mapping.source.start.character
                else:
                    char_offset = pos.character

                # Apply to target
                target_line = mapping.target.start.line + line_offset
                if line_offset == 0:
                    target_char = mapping.target.start.character + char_offset
                else:
                    target_char = char_offset

                return Position(line=target_line, character=target_char)

        # Fall back to line mapping
        if pos.line in self._hyper_to_python_lines:
            return Position(
                line=self._hyper_to_python_lines[pos.line],
                character=pos.character
            )

        return None

    def python_to_hyper(self, pos: Position) -> Optional[Position]:
        """Convert a position in generated Python to the corresponding position in .hyper."""
        # First check exact mappings
        for mapping in self.mappings:
            if self._in_range(pos, mapping.target):
                line_offset = pos.line - mapping.target.start.line
                if line_offset == 0:
                    char_offset = pos.character - mapping.target.start.character
                else:
                    char_offset = pos.character

                source_line = mapping.source.start.line + line_offset
                if line_offset == 0:
                    source_char = mapping.source.start.character + char_offset
                else:
                    source_char = char_offset

                return Position(line=source_line, character=source_char)

        # Fall back to line mapping
        if pos.line in self._python_to_hyper_lines:
            return Position(
                line=self._python_to_hyper_lines[pos.line],
                character=pos.character
            )

        return None

    def _in_range(self, pos: Position, range: Range) -> bool:
        """Check if position is within range."""
        if pos.line < range.start.line or pos.line > range.end.line:
            return False
        if pos.line == range.start.line and pos.character < range.start.character:
            return False
        if pos.line == range.end.line and pos.character > range.end.character:
            return False
        return True
