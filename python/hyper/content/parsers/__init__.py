"""File format parsers with protocol-based extensibility."""

from pathlib import Path
from typing import Any, Protocol


class Parser(Protocol):
    """Protocol for file format parsers."""

    @staticmethod
    def can_parse(path: Path) -> bool:
        """Return True if this parser can handle the given file."""
        ...

    @staticmethod
    def parse(content: bytes, target_type: type | None = None) -> Any:
        """Parse file content and return dict/list or typed object.

        Args:
            content: Raw file bytes to parse
            target_type: Optional type hint for direct parsing optimization
                         (e.g., msgspec can parse JSON directly to Struct)
        """
        ...


# Import and register all parsers
from hyper.content.parsers.json import MsgspecJsonParser, StdlibJsonParser  # noqa: E402
from hyper.content.parsers.markdown import MarkdownParser  # noqa: E402
from hyper.content.parsers.toml import TomlParser  # noqa: E402
from hyper.content.parsers.yaml import YamlParser  # noqa: E402

# Parser registry - order matters! First match wins
PARSERS: list[type[Parser]] = [
    MsgspecJsonParser,  # Try msgspec first for JSON (fastest)
    StdlibJsonParser,  # Fallback to stdlib JSON
    YamlParser,  # YAML support
    TomlParser,  # TOML support
    MarkdownParser,  # Markdown with frontmatter
]


def parse_file(
    path: Path, target_type: type | None = None, content: bytes | None = None
) -> Any:
    """Parse a file using the first available parser.

    Args:
        path: Path to file (used to determine parser and as context)
        target_type: Optional type to parse directly into (msgspec optimization)
        content: Optional pre-loaded bytes (if None, reads from path)

    Returns:
        Parsed data (dict/list or typed object if target_type supported)

    Raises:
        ValueError: If no parser can handle this file type
    """
    if content is None:
        content = path.read_bytes()

    for parser in PARSERS:
        if parser.can_parse(path):
            # Try parsing with target_type if parser supports it
            try:
                return parser.parse(content, target_type)
            except TypeError:
                # Parser doesn't accept target_type, try without it
                return parser.parse(content)

    raise ValueError(
        f"Unsupported file extension '{path.suffix}' in {path.name}. "
        f"Supported formats: .json, .yaml, .yml, .toml, .md, .markdown"
    )
