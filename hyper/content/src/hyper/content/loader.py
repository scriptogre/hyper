import glob
from pathlib import Path
from typing import Any, TypeVar, get_args, get_origin, overload

from hyper.content.converters import convert
from hyper.content.parsers import parse_file

T = TypeVar("T")


def deep_merge(base: dict, override: dict) -> dict:
    """Recursively merge two dictionaries."""
    result = base.copy()
    for key, value in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = deep_merge(result[key], value)
        else:
            result[key] = value
    return result


def inject_metadata(data: Any, path: Path) -> None:
    """Inject metadata into loaded data (only for dicts)."""
    if isinstance(data, dict):
        if "id" not in data:
            # Use relative path from cwd, with extension removed
            # E.g., "docs/guides/getting-started.md" → "docs/guides/getting-started"
            try:
                # Resolve both paths to handle symlinks (e.g., /tmp → /private/tmp on macOS)
                resolved_path = path.resolve()
                resolved_cwd = Path.cwd().resolve()
                rel_path = resolved_path.relative_to(resolved_cwd)
                # Remove extension and convert to forward slashes
                data["id"] = str(rel_path.with_suffix(''))
            except ValueError:
                # If path is not relative to cwd, just use stem
                data["id"] = path.stem
    elif isinstance(data, list):
        for item in data:
            if isinstance(item, dict) and "_source" not in item:
                item["_source"] = path.name


# ==========================================
# Core Loader - Library Agnostic
# ==========================================
class Loader:
    """
    A smart loader that's completely library-agnostic.
    Works with msgspec, pydantic, or any other library via converters.
    """

    @overload
    def __call__(
        self, pattern: str, model: type[list[T]], *, merge: str = "shallow"
    ) -> list[T]: ...

    @overload
    def __call__(
        self, pattern: str, model: type[T], *, merge: str = "shallow"
    ) -> T: ...

    @overload
    def __call__(self, pattern: str, *, merge: str = "shallow") -> Any: ...

    def __call__(self, pattern: str, model: Any = None, *, merge: str = "shallow") -> Any:
        return self._execute(pattern, model, merge)

    def __getitem__(self, model: type[T]):
        def wrapper(pattern: str, *, merge: str = "shallow") -> T:
            return self._execute(pattern, model, merge)
        return wrapper

    def _execute(self, pattern: str, type_hint: Any = None, merge: str = "shallow") -> Any:
        origin = get_origin(type_hint)

        # No type hint? Return raw data (no conversion)
        if type_hint is None:
            paths = [Path(p) for p in glob.glob(pattern, recursive=True)]
            if not paths and not glob.has_magic(pattern):
                raise FileNotFoundError(f"File not found: {pattern}")
            if len(paths) == 1:
                return parse_file(paths[0])
            return [parse_file(p) for p in paths]

        # Determine collection vs singleton
        is_collection = origin is list
        model_cls = get_args(type_hint)[0] if is_collection else type_hint

        # Extract hooks from Meta if present
        before_parse_hook = None
        after_parse_hook = None
        after_load_hook = None
        if hasattr(model_cls, 'Meta'):
            before_parse_hook = getattr(model_cls.Meta, 'before_parse', None)
            after_parse_hook = getattr(model_cls.Meta, 'after_parse', None)
            after_load_hook = getattr(model_cls.Meta, 'after_load', None)

        # Load files
        paths = sorted([Path(p) for p in glob.glob(pattern, recursive=True)])
        if not paths and not glob.has_magic(pattern):
            raise FileNotFoundError(f"File not found: {pattern}")

        # Parse to dict/list, then convert
        raw_items = []
        for path in paths:
            # Apply before_parse hook if present
            raw_bytes = path.read_bytes()
            if before_parse_hook:
                raw_bytes = before_parse_hook(path, raw_bytes)

            # Singleton optimization: For single file, try direct parsing to target type
            if not is_collection and len(paths) == 1:
                try:
                    result = parse_file(path, target_type=model_cls, content=raw_bytes)
                    # If parser returned the target type directly (optimization worked)
                    if not isinstance(result, (dict, list)):
                        inject_metadata(result, path)
                        # For msgspec direct parse, __init__ wasn't called, so run after_load manually
                        if after_load_hook:
                            result = after_load_hook(result)
                        return result
                except (TypeError, AttributeError):
                    # Optimization didn't work, fall through to normal parsing
                    pass

            # Parse the (possibly modified) bytes
            content = parse_file(path, content=raw_bytes)

            # Apply after_parse hook if present
            if after_parse_hook:
                content = after_parse_hook(path, content)

            inject_metadata(content, path)
            raw_items.append(content)

        if not raw_items and is_collection:
            return []
        if not raw_items:
            raise ValueError(f"No data found for {pattern}")

        # Collection mode: flatten and convert
        if is_collection:
            flat_list = []
            for item in raw_items:
                if isinstance(item, list):
                    flat_list.extend(item)
                else:
                    flat_list.append(item)
            result = convert(flat_list, model_cls, is_list=True)

            # Apply after_load hook to each item (file-based loading only)
            if after_load_hook:
                result = [after_load_hook(item) for item in result]

            return result

        # Singleton mode: merge and convert
        else:
            if len(raw_items) > 1:
                # Merge multiple files according to strategy
                final_data = raw_items[0]
                if not isinstance(final_data, dict):
                    raise ValueError(
                        f"Cannot merge lists into a Singleton {model_cls.__name__}"
                    )

                for item in raw_items[1:]:
                    if not isinstance(item, dict):
                        raise ValueError(
                            f"Cannot merge lists into a Singleton {model_cls.__name__}"
                        )
                    if merge == "deep":
                        final_data = deep_merge(final_data, item)
                    else:  # shallow
                        final_data.update(item)
            else:
                final_data = raw_items[0]
                if isinstance(final_data, list):
                    raise ValueError(
                        f"File contains a list, but {model_cls.__name__} (Singleton) was requested."
                    )

            result = convert(final_data, model_cls, is_list=False)

            # Apply after_load hook (file-based loading only)
            if after_load_hook:
                result = after_load_hook(result)

            return result


# Global loader instance
load = Loader()
