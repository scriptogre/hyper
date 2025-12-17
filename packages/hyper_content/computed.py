"""Computed field decorator for lazy-evaluated derived properties."""

from functools import wraps
from typing import Any, Callable, TypeVar


T = TypeVar('T')


def computed(func: Callable[[Any], T]) -> property:
    """Decorator for computed/derived fields that are lazily evaluated.

    Computed fields are cached after first access for performance.

    Example:
        class BlogPost(Collection):
            body: str

            @computed
            def word_count(self) -> int:
                return len(self.body.split())

            @computed
            def reading_time(self) -> str:
                minutes = self.word_count // 200
                return f"{minutes} min read"

    The decorator converts the method into a cached property that:
    - Computes value on first access
    - Caches result for subsequent accesses
    - Can reference other computed fields
    """
    cache_attr = f'_computed_cache_{func.__name__}'

    @wraps(func)
    def getter(self):
        # Check if already computed
        if not hasattr(self, cache_attr):
            # Compute and cache
            result = func(self)
            setattr(self, cache_attr, result)
        return getattr(self, cache_attr)

    return property(getter)
