"""Base mixin classes for Singleton and Collection models."""

from typing import ClassVar, Self

from hyper_content.loader import load


class SingletonMixin:
    """Mixin providing .load() method for singleton models."""

    class Meta:
        pattern: ClassVar[str]

    @classmethod
    def load(cls) -> Self:
        # Check for URL-based loading
        if hasattr(cls, "Meta") and hasattr(cls.Meta, "url"):
            from hyper_content.loaders import load_from_url
            data = load_from_url(cls.Meta.url)
            # URL loader returns list, take first item for Singleton
            if not data:
                raise ValueError(f"No data returned from {cls.Meta.url}")
            return cls(**data[0])

        # File-based loading
        if not hasattr(cls, "Meta") or not hasattr(cls.Meta, "pattern"):
            raise ValueError(f"Missing Meta.pattern or Meta.url on {cls.__name__}")
        return load(cls.Meta.pattern, cls)


class CollectionMixin:
    """Mixin providing .load() method for collection models."""

    class Meta:
        pattern: ClassVar[str]

    @classmethod
    def load(cls) -> list[Self]:
        # Check for URL-based loading
        if hasattr(cls, "Meta") and hasattr(cls.Meta, "url"):
            from hyper_content.loaders import load_from_url
            data = load_from_url(cls.Meta.url)
            return [cls(**item) for item in data]

        # File-based loading
        if not hasattr(cls, "Meta") or not hasattr(cls.Meta, "pattern"):
            raise ValueError(f"Missing Meta.pattern or Meta.url on {cls.__name__}")
        return load(cls.Meta.pattern, list[cls])
