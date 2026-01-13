"""Markdown support for content collections."""

from dataclasses import dataclass
from functools import cached_property
from typing import Any

try:
    import pydantic  # noqa: F401

    HAS_PYDANTIC = True
except ImportError:
    HAS_PYDANTIC = False

try:
    import msgspec

    HAS_MSGSPEC = True
except ImportError:
    HAS_MSGSPEC = False
    msgspec = None


@dataclass
class Heading:
    """A heading extracted from markdown content."""

    level: int  # 1-6 (h1-h6)
    text: str  # Raw text content
    slug: str  # Anchor ID for linking


class TOC:
    """Table of contents structure."""

    def __init__(self, headings: list[Heading]):
        self.headings = headings

    def nested(self) -> list[dict[str, Any]]:
        """Return nested hierarchical structure."""
        if not self.headings:
            return []

        result: list[dict[str, Any]] = []
        stack: list[dict[str, Any]] = []

        for heading in self.headings:
            node = {"heading": heading, "children": []}

            while stack and stack[-1]["heading"].level >= heading.level:
                stack.pop()

            if stack:
                stack[-1]["children"].append(node)
            else:
                result.append(node)

            stack.append(node)

        return result


class _MarkdownMixin:
    """Internal mixin providing markdown-specific properties."""

    # These will be injected by the loader
    id: str
    body: str
    html: str

    @property
    def slug(self) -> str:
        """URL-friendly slug auto-generated from filename."""
        model_class = self.__class__
        if hasattr(model_class, "model_fields") and "slug" in model_class.model_fields:
            if "__dict__" in dir(self) and "slug" in self.__dict__:
                return self.__dict__["slug"]
            try:
                dumped = self.model_dump()
                if "slug" in dumped:
                    return dumped["slug"]
            except Exception:  # nosec B110 - intentional fallback for pydantic compatibility
                pass

        return str(self.id) if hasattr(self, "id") else "unknown"

    @cached_property
    def headings(self) -> list[Heading]:
        """Extract headings from markdown content."""
        import re

        headings_list = []
        atx_pattern = re.compile(r"^(#{1,6})\s+(.+?)(?:\s+#{1,6})?\s*$", re.MULTILINE)

        for match in atx_pattern.finditer(self.body):
            level = len(match.group(1))
            text = match.group(2).strip()
            slug = _slugify(text)
            headings_list.append(Heading(level, text, slug))

        return headings_list

    @cached_property
    def toc(self) -> TOC:
        """Generate table of contents from headings."""
        return TOC(self.headings)


def _slugify(text: str) -> str:
    """Convert text to URL-friendly slug."""
    import re

    # Remove markdown formatting
    text = re.sub(r"\[([^\]]+)\]\([^\)]+\)", r"\1", text)  # Links
    text = re.sub(r"[*_~`]", "", text)  # Bold, italic, code

    # Convert to lowercase and replace spaces/special chars with hyphens
    text = text.lower()
    text = re.sub(r"[^\w\s-]", "", text)
    text = re.sub(r"[-\s]+", "-", text)
    text = text.strip("-")

    return text


# Convenience classes that combine Collection/Singleton with Markdown
from hyper.content._mixins import CollectionMixin, SingletonMixin  # noqa: E402

# For msgspec, create base Structs with markdown fields pre-defined
if HAS_MSGSPEC:

    class _MarkdownStructBase(msgspec.Struct):
        """Base struct with markdown fields for msgspec users."""

        id: str
        body: str
        html: str


# Auto-dataclass mixins that apply @dataclass when needed
class _AutoDataclassCollectionMixin:
    """Mixin that auto-applies @dataclass for non-pydantic/non-msgspec classes."""

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)

        # Check if pydantic is being used
        has_pydantic = any(hasattr(base, "model_fields") for base in cls.__mro__)

        # If not pydantic and not already a dataclass, apply @dataclass in-place
        if not has_pydantic and not hasattr(cls, "__dataclass_fields__"):
            from dataclasses import dataclass as apply_dataclass

            # Apply dataclass and copy methods back to original class
            dc_cls = apply_dataclass(cls)
            cls.__init__ = dc_cls.__init__
            cls.__repr__ = dc_cls.__repr__
            cls.__eq__ = dc_cls.__eq__
            cls.__dataclass_fields__ = dc_cls.__dataclass_fields__
            if hasattr(dc_cls, "__dataclass_params__"):
                cls.__dataclass_params__ = dc_cls.__dataclass_params__


class _AutoDataclassSingletonMixin:
    """Mixin that auto-applies @dataclass for non-pydantic/non-msgspec classes."""

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)

        # Check if pydantic is being used
        has_pydantic = any(hasattr(base, "model_fields") for base in cls.__mro__)

        # If not pydantic and not already a dataclass, apply @dataclass in-place
        if not has_pydantic and not hasattr(cls, "__dataclass_fields__"):
            from dataclasses import dataclass as apply_dataclass

            # Apply dataclass and copy methods back to original class
            dc_cls = apply_dataclass(cls)
            cls.__init__ = dc_cls.__init__
            cls.__repr__ = dc_cls.__repr__
            cls.__eq__ = dc_cls.__eq__
            cls.__dataclass_fields__ = dc_cls.__dataclass_fields__
            if hasattr(dc_cls, "__dataclass_params__"):
                cls.__dataclass_params__ = dc_cls.__dataclass_params__


class _MarkdownCollectionDescriptor:
    """Descriptor that adapts to pydantic, msgspec, or dataclass."""

    def __instancecheck__(self, instance):
        """Support isinstance(obj, MarkdownCollection)."""
        return isinstance(instance, CollectionMixin) and isinstance(
            instance, _MarkdownMixin
        )

    def __subclasscheck__(self, subclass):
        """Support issubclass(cls, MarkdownCollection)."""
        return issubclass(subclass, CollectionMixin) and issubclass(
            subclass, _MarkdownMixin
        )

    def __mro_entries__(self, bases):
        """Called when this instance is used as a base class."""
        # Check if msgspec.Struct is in the bases
        if HAS_MSGSPEC:
            struct_idx = None
            for i, b in enumerate(bases):
                if b is msgspec.Struct or (
                    isinstance(b, type) and issubclass(b, msgspec.Struct)
                ):
                    struct_idx = i
                    break

            if struct_idx is not None:
                # Check if MarkdownCollection comes AFTER msgspec.Struct (wrong order)
                # We are always at the end of bases when __mro_entries__ is called
                if struct_idx == 0:
                    raise TypeError(
                        "MarkdownCollection must come before msgspec.Struct in the base class list.\n"
                        "Use: class YourClass(MarkdownCollection, msgspec.Struct)\n"
                        "Not: class YourClass(msgspec.Struct, MarkdownCollection)"
                    )

                # Return Struct base with fields + mixins
                return CollectionMixin, _MarkdownMixin, _MarkdownStructBase

        # For pydantic/dataclass, return mixins only
        return _AutoDataclassCollectionMixin, CollectionMixin, _MarkdownMixin


class _MarkdownSingletonDescriptor:
    """Descriptor that adapts to pydantic, msgspec, or dataclass."""

    def __instancecheck__(self, instance):
        """Support isinstance(obj, MarkdownSingleton)."""
        return isinstance(instance, SingletonMixin) and isinstance(
            instance, _MarkdownMixin
        )

    def __subclasscheck__(self, subclass):
        """Support issubclass(cls, MarkdownSingleton)."""
        return issubclass(subclass, SingletonMixin) and issubclass(
            subclass, _MarkdownMixin
        )

    def __mro_entries__(self, bases):
        """Called when this instance is used as a base class."""
        # Check if msgspec.Struct is in the bases
        if HAS_MSGSPEC:
            struct_idx = None
            for i, b in enumerate(bases):
                if b is msgspec.Struct or (
                    isinstance(b, type) and issubclass(b, msgspec.Struct)
                ):
                    struct_idx = i
                    break

            if struct_idx is not None:
                # Check if MarkdownSingleton comes AFTER msgspec.Struct (wrong order)
                if struct_idx == 0:
                    raise TypeError(
                        "MarkdownSingleton must come before msgspec.Struct in the base class list.\n"
                        "Use: class YourClass(MarkdownSingleton, msgspec.Struct)\n"
                        "Not: class YourClass(msgspec.Struct, MarkdownSingleton)"
                    )

                # Return Struct base with fields + mixins
                return SingletonMixin, _MarkdownMixin, _MarkdownStructBase

        # For pydantic/dataclass, return mixins only
        return _AutoDataclassSingletonMixin, SingletonMixin, _MarkdownMixin


# Create singleton instances to use as base "classes"
MarkdownCollection = _MarkdownCollectionDescriptor()
MarkdownSingleton = _MarkdownSingletonDescriptor()
