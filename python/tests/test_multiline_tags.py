from __future__ import annotations

import pytest

pytest.importorskip("hyperhtml._native")

from hyperhtml import _native


def compile_template(source: str, **globals_):
    generated = _native.transpile(source, "Template.hyper")
    namespace = dict(globals_)
    exec(compile(generated, "Template.hyper", "exec"), namespace)
    return namespace["Template"]


def test_multiline_html_attributes_do_not_render_formatting_whitespace():
    template = compile_template(
        """active: bool
attrs: dict
---
<div
    id="profile"
    class={["card", {"active": active}]}
    {**attrs}
>
    Content
</div>
"""
    )

    assert template(active=True, attrs={"data-role": "admin"}) == (
        '<div id="profile" class="card active" data-role="admin">Content</div>'
    )


def test_multiline_quoted_attribute_preserves_content_whitespace():
    template = compile_template(
        """---
<button
    _="
        on click
            toggle .active
    "
>
    Toggle
</button>
"""
    )

    assert template() == (
        '<button _="\n'
        "        on click\n"
        "            toggle .active\n"
        '    ">Toggle</button>'
    )


def test_multiline_expression_uses_python_bracket_depth():
    template = compile_template(
        """selected: bool
---
<div
    class={
        [
            "card",
            {"selected": selected},
        ]
    }
>
    Content
</div>
"""
    )

    assert template(selected=True) == '<div class="card selected">Content</div>'


def test_multiline_self_closing_element():
    template = compile_template(
        """src: str
alt: str
---
<img
    src={src}
    alt={alt}
/>
"""
    )

    assert template(src="/avatar.png", alt="Ada") == '<img src="/avatar.png" alt="Ada">'


def test_multiline_component_call_with_content():
    from hyperhtml import component

    @component
    def Card(*, title: str, selected: bool = False, content=None):
        class_name = " selected" if selected else ""
        yield f'<article class="card{class_name}"><h2>{title}</h2>'
        if content is not None:
            yield from content
        yield "</article>"

    template = compile_template(
        """Card: object
title: str
---
<{Card}
    title={title}
    selected
>
    <p>Content</p>
</{Card}>
""",
        Card=Card,
    )

    assert template(Card=Card, title="Profile") == (
        '<article class="card selected"><h2>Profile</h2><p>Content</p></article>'
    )


def test_statement_inside_multiline_tag_has_focused_error():
    with pytest.raises(Exception) as caught:
        _native.transpile(
            """active: bool
---
<div
    if active:
        class="active"
>
""",
            "Template.hyper",
        )

    message = str(caught.value)
    assert "Python statements cannot appear inside an opening tag" in message
    assert '<div class={"active" if active else None}>' in message


def test_unterminated_multiline_tag_has_focused_error():
    with pytest.raises(Exception) as caught:
        _native.transpile(
            """---
<div
    class="card"
""",
            "Template.hyper",
        )

    assert "unclosed opening tag" in str(caught.value).lower()


def test_duplicate_attributes_across_lines_are_rejected():
    with pytest.raises(Exception) as caught:
        _native.transpile(
            """---
<div
    class="card"
    class="active"
>
""",
            "Template.hyper",
        )

    assert "set twice" in str(caught.value).lower()
