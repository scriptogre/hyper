"""JSON parsers."""

from pathlib import Path
from typing import Any


class MsgspecJsonParser:
    """Fast JSON parser using msgspec with direct-to-struct optimization."""

    @staticmethod
    def can_parse(path: Path) -> bool:
        try:
            import msgspec  # noqa: F401

            return path.suffix.lower() == ".json"
        except ImportError:
            return False

    @staticmethod
    def parse(content: bytes, target_type: type | None = None) -> Any:
        import msgspec.json

        # Optimization: If target_type is a msgspec.Struct, decode directly
        if target_type is not None:
            try:
                import msgspec

                if isinstance(target_type, type) and issubclass(
                    target_type, msgspec.Struct
                ):
                    return msgspec.json.decode(content, type=target_type)
            except (TypeError, AttributeError):
                pass

        # Default: decode to dict/list
        return msgspec.json.decode(content)


class StdlibJsonParser:
    """Fallback JSON parser using standard library."""

    @staticmethod
    def can_parse(path: Path) -> bool:
        return path.suffix.lower() == ".json"

    @staticmethod
    def parse(content: bytes, target_type: type | None = None) -> Any:
        import json

        # stdlib json always returns dict/list, no optimization possible
        return json.loads(content)
