"""Type-safe template system using Python 3.14 t-strings.

Templates are Python files with props (type hints) and slots ({...}).
Props are HTML-escaped; children are trusted.
"""

import sys
import importlib
import importlib.util
from pathlib import Path

from hyper.templates.component import Template, load_template, render  # noqa: E402
from hyper.templates.errors import (  # noqa: E402
    TemplateCompileError,
    TemplateNotFoundError,
    PropValidationError,
    SlotError,
    TemplateError,
)
from hyper.templates.loader import Prop, extract_props  # noqa: E402
from hyper.templates.slots import slot  # noqa: E402
from hyper.templates import context  # noqa: E402


class _TemplateLoader:
    """Custom loader that returns a Template instead of a module."""

    def __init__(self, template_path: "Path"):
        self.template_path = template_path

    def create_module(self, spec):
        """Return the Template as the module."""
        return load_template(self.template_path)

    def exec_module(self, module):
        """Nothing to execute - module is already the Template."""
        pass


class _TemplateFinder:
    """Custom import finder that intercepts template imports.

    This is needed because Python's import system finds .py files as submodules
    before calling __getattr__. This finder intercepts those imports and returns
    Template instances instead.
    """

    def __init__(self, package_name: str, package_dir: "Path", is_excluded):
        self.package_name = package_name
        self.package_dir = package_dir
        self.is_excluded = is_excluded

    def find_spec(self, fullname: str, path=None, target=None):
        """Check if this import should be handled by us."""
        if not fullname.startswith(self.package_name + "."):
            return None

        # Get the name after the package prefix
        name = fullname[len(self.package_name) + 1 :]

        # Only handle direct children (no dots)
        if "." in name:
            return None

        # Check if there's a matching template file
        possible = self._get_possible_filenames(name)
        for filename in possible:
            if not self.is_excluded(filename):
                template_path = self.package_dir / filename
                if template_path.exists():
                    loader = _TemplateLoader(template_path)
                    return importlib.util.spec_from_loader(
                        fullname, loader, origin=str(template_path)
                    )

        return None

    def _get_possible_filenames(self, name: str) -> list:
        """Get possible filenames for a template name."""
        if name[0].isupper():
            snake = "".join(
                ["_" + c.lower() if c.isupper() else c for c in name]
            ).lstrip("_")
            return [f"{name}.py", f"{name.lower()}.py", f"{snake}.py"]
        else:
            pascal = "".join(w.capitalize() for w in name.split("_"))
            return [f"{name}.py", f"{pascal}.py"]


class _EnableTemplates:
    """Magic enabler that activates on import.

    Usage - just import it:
        # app/templates/__init__.py
        from hyper import enable_templates

    Usage - with exclusions:
        # app/templates/__init__.py
        from hyper import enable_templates
        enable_templates(exclude=["old_*.py"])
    """

    def __init__(self):
        self._enabled_modules = set()
        self._finders = {}

    def __call__(self, exclude=None):
        """Call explicitly with exclusions."""
        self._do_enable(exclude=exclude)
        return self

    def _do_enable(self, exclude=None):
        import inspect
        from pathlib import Path
        from fnmatch import fnmatch

        exclude = exclude or []

        # Get the caller - walk back to find the importing module
        # Skip our own frames and internal Python frames
        frame = inspect.currentframe().f_back
        while frame:
            module_name = frame.f_globals.get("__name__", "")
            # Skip hyper.templates, hyper, and importlib frames
            if (
                module_name.startswith("hyper.templates")
                or module_name.startswith("hyper")
                or module_name.startswith("importlib")
                or module_name.startswith("_frozen_importlib")
            ):
                frame = frame.f_back
                continue
            break

        if not frame:
            return

        # Skip if caller doesn't have a file (e.g., __main__ in a REPL)
        if "__file__" not in frame.f_globals:
            return

        caller_module_name = frame.f_globals["__name__"]

        # Don't enable twice
        if caller_module_name in self._enabled_modules:
            return

        self._enabled_modules.add(caller_module_name)

        caller_file = Path(frame.f_globals["__file__"])
        package_dir = caller_file.parent

        caller_module = sys.modules[caller_module_name]

        def is_excluded(filename):
            return any(fnmatch(filename, p) for p in exclude)

        # Install custom import finder
        finder = _TemplateFinder(caller_module_name, package_dir, is_excluded)
        self._finders[caller_module_name] = finder
        sys.meta_path.insert(0, finder)
        # Invalidate caches so our finder takes precedence
        importlib.invalidate_caches()

        # Also set up __getattr__ for attribute access (e.g., templates.Button)
        def __getattr__(name):
            # Map import name to possible filenames
            possible = finder._get_possible_filenames(name)

            for filename in possible:
                if not is_excluded(filename):
                    path = package_dir / filename
                    if path.exists():
                        tmpl = load_template(path)
                        setattr(caller_module, name, tmpl)
                        return tmpl

            raise AttributeError(f"No template '{name}' in {caller_module_name}")

        caller_module.__getattr__ = __getattr__


_enable_templates_instance = _EnableTemplates()


# Module-level __getattr__ to intercept access to enable_templates
def __getattr__(name):
    if name == "enable_templates":
        # Trigger the enabling automatically when accessed
        _enable_templates_instance._do_enable()
        return _enable_templates_instance
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")


__all__ = [
    "enable_templates",
    # Errors
    "TemplateError",
    "PropValidationError",
    "TemplateNotFoundError",
    "TemplateCompileError",
    "SlotError",
    # Internal
    "load_template",
    "render",
    "Template",
    "Prop",
    "extract_props",
    "context",
    "slot",
]
