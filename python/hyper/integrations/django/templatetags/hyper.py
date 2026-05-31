"""Django template tag for invoking Hyper components from DTE templates.

Usage:

    {% load hyper %}    {# omit if registered as a builtin in TEMPLATES OPTIONS #}

    {# no slots: self-close with a trailing slash (mind the space) #}
    {% hyper Greeting name=user.first_name / %}

    {# slots: body fills the default slot, {% slot name %} fills a named one #}
    {% hyper Card title="Pricing" %}
        <p>Three tiers, no surprises.</p>
        {% slot actions %}<a href="/buy">Buy now</a>{% endslot %}
    {% endhyper %}

    {# forward a dict of props; explicit kwargs first, **spread last #}
    {% hyper Card title="Pricing" **card_props %}…{% endhyper %}

    {# dotted-path fallback for components not in the context #}
    {% hyper "myapp.components.Sidebar" user=user / %}

The component is either a callable already in the template context (populate via
``hyper.integrations.django.context_processors.components``) or a dotted import
path. Output is marked safe; Hyper components escape their own arguments.
"""

from __future__ import annotations

import importlib
import re

from django import template
from django.utils.safestring import mark_safe

register = template.Library()

# Splits one tag bit into an optional `name=` prefix and the value expression,
# matching Django's own keyword-argument parsing.
_kwarg_re = re.compile(r"(?:([\w-]+)=)?(.+)", re.DOTALL)


def _resolve(target):
    """Accept a callable directly, or a 'pkg.mod.Name' string to import."""
    if callable(target):
        return target
    if isinstance(target, str):
        if "." not in target:
            raise template.TemplateSyntaxError(
                f"{{% hyper %}}: {target!r} is a bare string, not a dotted "
                "import path. Either pass a component from context (set up "
                "the components context processor) or write the full path "
                "like 'myapp.components.Sidebar'."
            )
        module_path, _, attr = target.rpartition(".")
        try:
            module = importlib.import_module(module_path)
        except ImportError as e:
            raise template.TemplateSyntaxError(
                f"{{% hyper %}}: could not import {module_path!r}: {e}"
            ) from e
        try:
            return getattr(module, attr)
        except AttributeError as e:
            raise template.TemplateSyntaxError(
                f"{{% hyper %}}: {module_path!r} has no attribute {attr!r}"
            ) from e
    raise template.TemplateSyntaxError(
        f"{{% hyper %}}: expected a component or dotted import path, "
        f"got {type(target).__name__}"
    )


def _parse_binders(parser, bits):
    """Turn the tag's argument bits into ordered binders.

    Each binder is one of ``("kw", name, expr)``, ``("spread", expr)``, or
    ``("pos", expr)``. Keeping them ordered lets ``**spread`` and explicit
    kwargs merge in source order at render time (rightmost wins, like Python).
    """
    binders = []
    for bit in bits:
        if bit.startswith("**"):
            binders.append(("spread", parser.compile_filter(bit[2:])))
            continue
        name, expr = _kwarg_re.match(bit).groups()
        if name:
            binders.append(("kw", name, parser.compile_filter(expr)))
        else:
            binders.append(("pos", parser.compile_filter(expr)))
    return binders


@register.tag("hyper")
def do_hyper(parser, token):
    bits = token.split_contents()[1:]  # drop 'hyper'
    if not bits:
        raise template.TemplateSyntaxError(
            "{% hyper %} requires a component, e.g. {% hyper Card title='x' / %}"
        )

    self_closing = bits[-1] == "/"  # {% hyper Card ... / %}
    if self_closing:
        bits = bits[:-1]

    target = parser.compile_filter(bits[0])
    binders = _parse_binders(parser, bits[1:])

    nodelist = template.NodeList()
    if not self_closing:
        nodelist = parser.parse(("endhyper",))  # may contain SlotNodes
        parser.delete_first_token()

    return HyperNode(target, binders, nodelist)


@register.tag("slot")
def do_slot(parser, token):
    bits = token.split_contents()
    if len(bits) != 2:
        raise template.TemplateSyntaxError(
            "{% slot %} takes exactly one name, e.g. {% slot actions %}"
        )
    name = bits[1]
    nodelist = parser.parse(("endslot",))
    parser.delete_first_token()
    return SlotNode(name, nodelist)


class SlotNode(template.Node):
    """A named slot. Renders to "" inline; HyperNode pulls it out structurally."""

    def __init__(self, name, nodelist):
        self.name = name
        self.nodelist = nodelist

    def render(self, context):
        return ""


class HyperNode(template.Node):
    def __init__(self, target, binders, nodelist):
        self.target = target
        self.binders = binders
        self.nodelist = nodelist

    def render(self, context):
        component = _resolve(self.target.resolve(context))
        if not callable(component):
            raise template.TemplateSyntaxError(
                f"{{% hyper %}}: {component!r} is not callable"
            )

        # Bind args, same rules as a Python call. A `kw` binds one name; a
        # `spread` binds every key in a dict. Either way, a name that's already
        # bound is a duplicate, and we raise just like Python and Jinja do.
        args, kwargs = [], {}
        for binder in self.binders:
            kind = binder[0]
            if kind == "pos":
                args.append(binder[1].resolve(context))
                continue

            if kind == "kw":
                incoming = {binder[1]: binder[2].resolve(context)}
            else:  # spread
                incoming = dict(binder[1].resolve(context))

            for key, value in incoming.items():
                if key in kwargs:
                    raise template.TemplateSyntaxError(
                        f"{{% hyper %}}: got multiple values for keyword argument {key!r}"
                    )
                kwargs[key] = value

        # Read slots structurally from the body: each top-level {% slot %} fills
        # its named slot, everything else fills the default slot.
        default_parts = []
        for node in self.nodelist:
            if isinstance(node, SlotNode):
                kwargs[f"_{node.name}_slot"] = [node.nodelist.render(context)]
            else:
                default_parts.append(node.render(context))

        default = "".join(default_parts)
        if default.strip():
            kwargs["_default_slot"] = [default]

        return mark_safe(str(component(*args, **kwargs)))
