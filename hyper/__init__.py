"""Hyper - Modern Python framework for content and templates."""

from hyper_content import (
    Collection,
    MarkdownCollection,
    MarkdownSingleton,
    Singleton,
    computed,
    load,
)
from hyper_templates import (
    Component,
    ComponentCompileError,
    ComponentNotFoundError,
    Prop,
    PropValidationError,
    SlotError,
    TemplateError,
    context,
    extract_props,
    load_component,
    render,
    slot,
)

__all__ = [
    # From hyper_content
    "load",
    "Singleton",
    "Collection",
    "MarkdownCollection",
    "MarkdownSingleton",
    "computed",
    # From hyper_templates
    "load_component",
    "render",
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
