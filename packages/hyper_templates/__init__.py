"""Type-safe template system using Python 3.14 t-strings.

Components are Python files with props (type hints) and slots ({...}).
Props are HTML-escaped; children are trusted.
"""

import sys
from importlib import import_module
from importlib.abc import MetaPathFinder
from importlib.machinery import ModuleSpec
from importlib.util import spec_from_file_location
from pathlib import Path


class ComponentImportFinder(MetaPathFinder):
    """Auto-enable component imports for */components/ packages.

    Intercepts imports like `from app.components import Button` and loads
    Button.py as a Component object instead of a regular Python module.
    """

    def find_spec(self, fullname: str, path, target=None) -> ModuleSpec | None:
        parts = fullname.split('.')

        if 'components' not in parts:
            return None

        try:
            comp_idx = parts.index('components')

            # Must be importing a specific component (not just the package)
            if comp_idx >= len(parts) - 1:
                return None

            component_name = parts[-1]
            package_path = '.'.join(parts[:comp_idx + 1])

            # Get or import the components package
            package = sys.modules.get(package_path)
            if not package:
                try:
                    package = import_module(package_path)
                except ImportError:
                    return None

            # Find the component file
            if not hasattr(package, '__path__'):
                return None

            for pkg_path in package.__path__:
                component_file = Path(pkg_path) / f"{component_name}.py"
                if component_file.exists():
                    # Create inline loader
                    class Loader:
                        def create_module(self, spec):
                            return None

                        def exec_module(self, module):
                            from hyper_templates.component import load_component
                            component = load_component(component_file)
                            sys.modules[module.__name__] = component

                    return spec_from_file_location(
                        fullname,
                        component_file,
                        loader=Loader(),
                        submodule_search_locations=None
                    )

            return None

        except (ValueError, IndexError, AttributeError):
            return None


# Auto-install the import hook when hyper.templates is imported
_component_finder = ComponentImportFinder()
if _component_finder not in sys.meta_path:
    sys.meta_path.insert(0, _component_finder)


from hyper_templates.component import Component, load_component, render
from hyper_templates.errors import (
    ComponentCompileError,
    ComponentNotFoundError,
    PropValidationError,
    SlotError,
    TemplateError,
)
from hyper_templates.loader import Prop, extract_props
from hyper_templates.slots import slot
from hyper_templates import context


__all__ = [
    "render",
    "load_component",
    "slot",
    "Component",
    "Prop",
    "extract_props",
    "context",
    "TemplateError",
    "PropValidationError",
    "ComponentNotFoundError",
    "ComponentCompileError",
    "SlotError",
]