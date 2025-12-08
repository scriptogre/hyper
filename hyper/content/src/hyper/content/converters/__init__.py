"""Data converters with protocol-based extensibility."""

from typing import Any, Protocol


class Converter(Protocol):
    """Protocol for data converters."""

    @staticmethod
    def can_convert(target_type: Any) -> bool:
        """Return True if this converter can handle the target type."""
        ...

    @staticmethod
    def convert_single(data: dict, target_type: type) -> Any:
        """Convert a single dict to the target type."""
        ...

    @staticmethod
    def convert_list(data: list[dict], target_type: type) -> list[Any]:
        """Convert a list of dicts to a list of the target type."""
        ...


# Import and register all converters
from hyper.content.converters.pydantic import PydanticConverter
from hyper.content.converters.msgspec import MsgspecConverter
from hyper.content.converters.dataclass import DataclassConverter
from hyper.content.converters.primitives import PrimitiveConverter

# Converter registry - order matters! First match wins
CONVERTERS: list[type[Converter]] = [
    PydanticConverter,    # Pydantic first (matches base class priority)
    MsgspecConverter,     # Msgspec second
    DataclassConverter,   # Standard library dataclasses
    PrimitiveConverter,   # Primitives (int, str, etc)
]


def convert(data: Any, target_type: type, is_list: bool) -> Any:
    """Convert data using registered converters.

    Args:
        data: Data to convert (dict or list[dict])
        target_type: Target type to convert to
        is_list: Whether converting a list or single item

    Returns:
        Converted data

    Raises:
        TypeError: If no converter can handle the target type
    """
    for converter_cls in CONVERTERS:
        if converter_cls.can_convert(target_type):
            if is_list:
                return converter_cls.convert_list(data, target_type)
            else:
                return converter_cls.convert_single(data, target_type)

    raise TypeError(
        f"No converter available for {target_type}. "
        f"Install msgspec or pydantic, or register a custom converter."
    )
