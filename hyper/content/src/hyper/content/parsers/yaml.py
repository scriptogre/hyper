"""YAML parser using PyYAML."""

from pathlib import Path
from typing import Any


class YamlParser:
    """YAML parser using PyYAML."""

    @staticmethod
    def can_parse(path: Path) -> bool:
        return path.suffix.lower() in (".yaml", ".yml")

    @staticmethod
    def parse(content: bytes, target_type: type | None = None) -> Any:
        try:
            import yaml
            # PyYAML always returns dict, no optimization possible
            return yaml.safe_load(content)
        except ImportError:
            raise ImportError(
                "YAML support requires PyYAML. Install with: uv add pyyaml"
            )
