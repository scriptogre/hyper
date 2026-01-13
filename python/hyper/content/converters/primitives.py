"""Primitive type converter."""

from typing import Any


class PrimitiveConverter:
    """Converter for primitive types (int, str, float, bool, etc)."""

    @staticmethod
    def can_convert(target_type: Any) -> bool:
        if not isinstance(target_type, type):
            return False
        return target_type in (int, str, float, bool, bytes)

    @staticmethod
    def convert_single(data: Any, target_type: type) -> Any:
        return target_type(data)

    @staticmethod
    def convert_list(data: list, target_type: type) -> list[Any]:
        return [target_type(item) for item in data]
