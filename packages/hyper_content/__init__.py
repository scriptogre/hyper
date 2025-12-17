"""Load structured content from files with optional validation.

Load files without validation:
    from hyper import load
    settings = load("settings.json")  # dict
    posts = load("posts/*.json")      # list[dict]

Load with validation by combining Singleton/Collection with your validation library:
    import pydantic
    from hyper import Singleton

    class Settings(Singleton, pydantic.BaseModel):
        theme: str
        version: int
        class Meta:
            pattern = "settings.json"

Or with msgspec:
    import msgspec
    from hyper import Collection, Markdown

    class BlogPost(Collection, Markdown, msgspec.Struct):
        title: str
        ...

Or with dataclass:
    from dataclasses import dataclass
    from hyper import Singleton

    @dataclass
    class Data(Singleton):
        value: int
        ...
"""

from hyper_content._mixins import CollectionMixin, SingletonMixin
from hyper_content.computed import computed
from hyper_content.loader import load
from hyper_content.markdown import MarkdownCollection, MarkdownSingleton

# Bare base classes - combine with pydantic.BaseModel, msgspec.Struct, or @dataclass
class Singleton(SingletonMixin):
    """Singleton model - combine with pydantic.BaseModel, msgspec.Struct, or @dataclass."""

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)

        # Check if pydantic or msgspec is being used
        has_pydantic = any(hasattr(base, 'model_fields') for base in cls.__mro__)

        # Check if msgspec.Struct is actually in the inheritance chain
        has_msgspec = False
        try:
            import msgspec
            has_msgspec = any(
                isinstance(base, type) and issubclass(base, msgspec.Struct)
                for base in cls.__mro__
            )
        except ImportError:
            pass

        # If not pydantic/msgspec and not already a dataclass, apply @dataclass in-place
        if not has_pydantic and not has_msgspec and not hasattr(cls, '__dataclass_fields__'):
            from dataclasses import dataclass as apply_dataclass

            # Apply dataclass and copy methods back to original class
            dc_cls = apply_dataclass(cls)
            cls.__init__ = dc_cls.__init__
            cls.__repr__ = dc_cls.__repr__
            cls.__eq__ = dc_cls.__eq__
            cls.__dataclass_fields__ = dc_cls.__dataclass_fields__
            if hasattr(dc_cls, '__dataclass_params__'):
                cls.__dataclass_params__ = dc_cls.__dataclass_params__


class Collection(CollectionMixin):
    """Collection model - combine with pydantic.BaseModel, msgspec.Struct, or @dataclass."""

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)

        # Check if pydantic or msgspec is being used
        has_pydantic = any(hasattr(base, 'model_fields') for base in cls.__mro__)

        # Check if msgspec.Struct is actually in the inheritance chain
        has_msgspec = False
        try:
            import msgspec
            has_msgspec = any(
                isinstance(base, type) and issubclass(base, msgspec.Struct)
                for base in cls.__mro__
            )
        except ImportError:
            pass

        # If not pydantic/msgspec and not already a dataclass, apply @dataclass in-place
        if not has_pydantic and not has_msgspec and not hasattr(cls, '__dataclass_fields__'):
            from dataclasses import dataclass as apply_dataclass

            # Apply dataclass and copy methods back to original class
            dc_cls = apply_dataclass(cls)
            cls.__init__ = dc_cls.__init__
            cls.__repr__ = dc_cls.__repr__
            cls.__eq__ = dc_cls.__eq__
            cls.__dataclass_fields__ = dc_cls.__dataclass_fields__
            if hasattr(dc_cls, '__dataclass_params__'):
                cls.__dataclass_params__ = dc_cls.__dataclass_params__

# Build __all__
__all__ = [
    "load",
    "Singleton",
    "Collection",
    "MarkdownCollection",
    "MarkdownSingleton",
    "computed",
]
