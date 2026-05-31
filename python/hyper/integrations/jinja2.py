"""Jinja2 integration for Hyper components.

Drop the extension into a Jinja env and Hyper components living in the env's
loader paths become available as Jinja globals automatically:

    env = Environment(loader=FileSystemLoader("templates"))
    env.add_extension("hyper.integrations.jinja2.HyperExtension")

    # templates/Greeting.hyper compiled to templates/Greeting.py
    # In any Jinja template:    {{ Greeting(name="Ada") }}

For components that don't live next to Jinja templates, use the escape hatch
the extension attaches to the env:

    env.register_components(myapp.components)
    env.register_components(SomeComponent, [Other, Another])

To fill slots, wrap the call in {% hyper %} and mark named slots with
{% slot name %}:

    {% hyper Card(title="Pricing") %}
        <p>Body fills the default slot.</p>
        {% slot actions %}<a href="/buy">Buy</a>{% endslot %}
    {% endhyper %}
"""

from __future__ import annotations

import warnings
from collections.abc import Iterable, Iterator
from pathlib import Path

from jinja2 import BaseLoader, ChoiceLoader, FileSystemLoader, nodes
from jinja2.ext import Extension

from hyper.integrations._discovery import discover_in_package, discover_in_path

__all__ = ["HyperExtension"]


class _Slot:
    """A parsed {% slot name %} block, holding its name and body nodes.

    Not a Jinja node (Jinja forbids custom node types). It's a transient marker:
    `_parse_hyper` pulls these out of a component body and turns them into
    {% set %}-style captures, so one never reaches code generation.
    """

    __slots__ = ("name", "body", "lineno")

    def __init__(self, name, body, lineno):
        self.name = name
        self.body = body
        self.lineno = lineno


def _loader_paths(loader: BaseLoader | None) -> Iterator[Path]:
    """Yield filesystem paths for a Jinja loader, walking ChoiceLoader recursively.

    Returns nothing for loaders we can't introspect (PackageLoader, DictLoader,
    FunctionLoader, etc.). Those projects use the explicit
    `env.register_components(...)` escape hatch instead.
    """
    if loader is None:
        return
    if isinstance(loader, FileSystemLoader):
        for p in loader.searchpath:
            yield Path(p)
    elif isinstance(loader, ChoiceLoader):
        for child in loader.loaders:
            yield from _loader_paths(child)


class HyperExtension(Extension):
    """Jinja2 extension: discovers and registers Hyper components.

    At init time, walks the environment's loader for `.hyper` files. For each,
    imports the sibling `.py` and registers every `@html`-decorated callable
    (those carry `__hyper__ = True`) into `env.globals` keyed by function name.

    Also attaches `env.register_components(*sources)` for components that
    don't live next to Jinja templates. Accepts packages, single callables, or
    iterables.
    """

    tags = {"hyper", "slot"}

    def __init__(self, environment):
        super().__init__(environment)
        environment.extend(register_components=self._register)
        self._auto_discover(environment)

    # --- {% hyper %} / {% slot %} --------------------------------------------

    def parse(self, parser):
        token = next(parser.stream)
        if token.value == "hyper":
            return self._parse_hyper(parser, token)
        return self._parse_slot(parser, token)

    def _parse_hyper(self, parser, token):
        """{% hyper Card(title=...) %}…{% endhyper %} → plain Jinja.

        Rewrites the block so each {% slot %} and the default body is captured
        into a {% set %}-style variable, then passed to the component as the
        reserved ``_*_slot`` kwarg. The slot HTML reaches the component as an
        argument at the call site, exactly as the Django tag passes it. No
        runtime side-channel, and the component itself stays slot-agnostic.
        """
        call = parser.parse_expression()
        if not isinstance(call, nodes.Call):
            parser.fail(
                "{% hyper %} expects a component call, e.g. "
                "{% hyper Card(title=...) %}",
                token.lineno,
            )

        # Track nesting on the per-parse parser (not on self, which is shared
        # across threads) so a stray {% slot %} can fail with a clear message.
        parser._hyper_depth = getattr(parser, "_hyper_depth", 0) + 1
        try:
            body = parser.parse_statements(("name:endhyper",), drop_needle=True)
        finally:
            parser._hyper_depth -= 1

        # Each capture is a {% set var %}…{% endset %}; the component reads it
        # as `_<name>_slot=[var]` (list-wrapped, since it does `yield from`).
        setups = []
        default_body = []
        for node in body:
            if not isinstance(node, _Slot):
                default_body.append(node)
                continue
            var = parser.free_identifier(node.lineno)
            setups.append(nodes.AssignBlock(nodes.Name(var.name, "store"), None, node.body))
            call.kwargs.append(
                nodes.Keyword(f"_{node.name}_slot", nodes.List([nodes.Name(var.name, "load")]))
            )

        # A whitespace-only body is no default slot at all, so the component's
        # own fallback shows (same rule the side-channel applied with .strip()).
        has_default = False
        for node in default_body:
            is_blank_text = isinstance(node, nodes.Output) and all(
                isinstance(item, nodes.TemplateData) and not item.data.strip()
                for item in node.nodes
            )
            if not is_blank_text:
                has_default = True
                break

        if has_default:
            var = parser.free_identifier(token.lineno)
            setups.append(nodes.AssignBlock(nodes.Name(var.name, "store"), None, default_body))
            call.kwargs.append(
                nodes.Keyword("_default_slot", nodes.List([nodes.Name(var.name, "load")]))
            )

        return [*setups, nodes.Output([call], lineno=token.lineno)]

    def _parse_slot(self, parser, token):
        """{% slot name %}…{% endslot %} → a transient _Slot, consumed by _parse_hyper."""
        if not getattr(parser, "_hyper_depth", 0):
            parser.fail("{% slot %} must appear inside a {% hyper %} block", token.lineno)
        name = parser.stream.expect("name").value
        body = parser.parse_statements(("name:endslot",), drop_needle=True)
        return _Slot(name, body, token.lineno)

    # --- internals -----------------------------------------------------------

    def _auto_discover(self, environment) -> None:
        paths = list(_loader_paths(environment.loader))
        if not paths:
            warnings.warn(
                "Hyper: Jinja loader has no introspectable filesystem paths. "
                "Components won't auto-register. Call "
                "env.register_components(...) explicitly.",
                stacklevel=3,
            )
            return
        seen: set[str] = set()
        for path in paths:
            for name, component in discover_in_path(path):
                if name in seen:
                    continue
                seen.add(name)
                environment.globals[name] = component

    def _register(self, *targets) -> None:
        env = self.environment
        for t in targets:
            if t is None:
                continue
            if hasattr(t, "__path__"):           # python package
                for name, component in discover_in_package(t):
                    env.globals[name] = component
            elif callable(t):                    # single component
                env.globals[t.__name__] = t
            elif isinstance(t, Iterable):        # iterable of any of the above
                self._register(*t)
            else:
                raise TypeError(
                    f"register_components: unsupported argument {t!r}. "
                    "Expected a package, a callable, or an iterable thereof."
                )
