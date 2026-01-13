"""Dataclass converter for standard library dataclasses."""

from typing import Any


class DataclassConverter:
    """Converter for standard library dataclasses (always available)."""

    @staticmethod
    def can_convert(target_type: Any) -> bool:
        if not isinstance(target_type, type):
            return False
        return hasattr(target_type, "__dataclass_fields__")

    @staticmethod
    def convert_single(data: dict, target_type: type) -> Any:
        # Filter to only fields that exist in the dataclass
        fields = set(target_type.__dataclass_fields__.keys())
        filtered = {k: v for k, v in data.items() if k in fields}
        return target_type(**filtered)

    @staticmethod
    def convert_list(data: list[dict], target_type: type) -> list[Any]:
        fields = set(target_type.__dataclass_fields__.keys())
        return [
            target_type(**{k: v for k, v in item.items() if k in fields})
            for item in data
        ]
