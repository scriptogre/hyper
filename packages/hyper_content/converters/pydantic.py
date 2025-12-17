"""Pydantic converter."""

from typing import Any


class PydanticConverter:
    """Converter for pydantic.BaseModel (install with: uv add hyper[pydantic])."""

    @staticmethod
    def can_convert(target_type: Any) -> bool:
        try:
            from pydantic import BaseModel

            return isinstance(target_type, type) and issubclass(target_type, BaseModel)
        except ImportError:
            return False

    @staticmethod
    def convert_single(data: dict, target_type: type) -> Any:
        return target_type.model_validate(data)

    @staticmethod
    def convert_list(data: list[dict], target_type: type) -> list[Any]:
        return [target_type.model_validate(item) for item in data]
