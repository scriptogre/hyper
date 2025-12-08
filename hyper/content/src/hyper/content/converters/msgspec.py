"""msgspec converter."""

from typing import Any


class MsgspecConverter:
    """Converter for msgspec.Struct (install with: uv add hyper[msgspec])."""

    @staticmethod
    def can_convert(target_type: Any) -> bool:
        try:
            import msgspec

            if not isinstance(target_type, type):
                return False
            return hasattr(msgspec, "Struct") and issubclass(
                target_type, msgspec.Struct
            )
        except ImportError:
            return False

    @staticmethod
    def convert_single(data: dict, target_type: type) -> Any:
        import msgspec

        return msgspec.convert(data, type=target_type)

    @staticmethod
    def convert_list(data: list[dict], target_type: type) -> list[Any]:
        import msgspec

        return msgspec.convert(data, type=list[target_type])
