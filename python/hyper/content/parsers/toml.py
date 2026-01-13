"""TOML parser using standard library."""

from pathlib import Path
from typing import Any


class TomlParser:
    """TOML parser using stdlib tomllib."""

    @staticmethod
    def can_parse(path: Path) -> bool:
        return path.suffix.lower() == ".toml"

    @staticmethod
    def parse(content: bytes, target_type: type | None = None) -> Any:
        import tomllib

        # tomllib always returns dict, no optimization possible
        return tomllib.loads(content.decode("utf-8"))
