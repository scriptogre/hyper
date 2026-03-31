from collections.abc import Iterable
from hyper import html, escape, render_attr


@html
def KitchenSink(_content: Iterable[str] | None = None, *, name: str, count: int = 0, is_active: bool = True, items: list = [], metadata: dict = {}, callback: object = None, style: str = "default", _header: Iterable[str] | None = None, _sidebar: Iterable[str] | None = None):
    # Kitchen sink: exercises every syntax construct for visual IDE smoke testing.
    # Open this file in JetBrains after any injection change and verify highlighting.

    # -- Standalone Python statements --

    result = name.upper()

    values = [x * 2 for x in range(count)]

    lookup = {k: v for k, v in metadata.items()}

    # -- Basic elements with expressions --
    yield f"""\
<div class="container" id="main-{escape(count)}" data-name="{escape(name)}">
    <h1>{escape(name)}</h1>
    <p>Count is {escape(count + 1)} and active is {escape(is_active)}</p>
    <span>{escape(f"Hello, {name}!")}</span>
</div>"""

    # -- Void / self-closing elements --
    yield f"""\
<div>
    <img src="/img/{escape(name)}.png" alt="{escape(name)}" />
    <input type="text" value="{escape(name)}"{render_attr("disabled", is_active)} />
</div>"""

    # -- All attribute kinds --
    yield f"""\
<div class="static" id="s-{escape(count)}" data-val="{escape(name)}"{render_attr("is_active", is_active)}{render_attr("metadata", metadata)}>
    Mixed attributes
</div>"""

    # -- If / elif / else --

    if is_active:
        yield """<span class="active">Active</span>"""
    elif count > 0:
        yield f"""<span class="partial">Partial ({escape(count)})</span>"""
    else:
        yield """<span class="inactive">Inactive</span>"""

    # -- For loop --

    for item in items:
        yield f"""<li class="item">{escape(item)}</li>"""

    # -- For loop with destructuring --

    for key, value in metadata.items():
        yield f"""\
<dt>{escape(key)}</dt>
    <dd>{escape(value)}</dd>"""

    # -- While loop --

    while count > 0:
        yield f"""\
<p>Counting down: {escape(count)}</p>
    """
        count = count - 1


    # -- Match / case --

    match style:
        case "bold":
            yield f"""\
<strong>{escape(name)}</strong>
    """
        case "italic":
            yield f"""\
<em>{escape(name)}</em>
    """
        case _:
            yield f"""<span>{escape(name)}</span>"""

    # -- Try / except / else / finally --

    try:
        yield f"""<span>{escape(metadata['key'])}</span>"""
    except KeyError as e:
        yield f"""<span>Missing: {escape(e)}</span>"""
    except ValueError:
        yield """<span>Bad value</span>"""
    else:
        yield """<span>Success</span>"""
    finally:
        yield """<span>Done</span>"""

    # -- With statement --

    with open("/dev/null") as f:
        yield f"""<pre>{escape(f.read())}</pre>"""

    # -- Decorators and nested definitions --

    @fragment
    def Badge(text: str, variant: str = "info"):
        yield f"""<span class="badge badge-{escape(variant)}">{escape(text)}</span>"""

    @cache
    @fragment
    def CachedList(entries: list):

        yield "<ul>"

        for entry in entries:
            yield f"""\
<li>{escape(entry)}</li>
        """

        yield "</ul>"


    # -- Using decorated functions --
    yield f"""\
{escape(Badge("New"))}
{escape(CachedList(items))}"""

    # -- Components --

    yield from Badge(text="Sale", variant="danger")

    # <{CachedList}>
    def _cached_list():
        yield """<p>Fallback content</p>"""
    yield from CachedList(_cached_list(), entries=items)
    # </{CachedList}>

    # -- Component with expression name --

    yield from callback()

    # -- Named slots with fallback --

    if _header is not None:
        yield from _header
    else:
        yield """<h2>Default Header</h2>"""

    if _sidebar is not None:
        yield from _sidebar
    else:
        yield """<nav>Default Nav</nav>"""

    # -- Default slot --

    if _content is not None:
        yield from _content

    # -- Deeply nested: components inside control flow inside elements --

    yield "<section>"

    if is_active:

        for item in items:

            yield from Badge(text=item)

            yield "<div class=\"wrapper\">"

            match item:
                case "special":

                    yield from CachedList(entries=[item, item])

                case _:
                    yield f"""\
<span>{escape(item)}</span>
                """

            yield "</div>"



    yield "</section>"

    # -- Adjacent expressions --
    yield f"""<p>{escape(name)}{escape(count)}{escape(is_active)}</p>"""

    # -- Escaped braces --
    yield """<code>Use {braces} in templates</code>"""

    # -- Nested elements --
    yield f"""\
<div>
    <ul>
        <li>
            <a href="/item/{escape(name)}">
                <span class="label">{escape(name)}</span>
            </a>
        </li>
    </ul>
</div>"""

    # -- Expressions with complex Python --
    yield f"""\
<p>{escape(", ".join(str(x) for x in items))}</p>
<p>{escape(name if is_active else "anonymous")}</p>
<p>{escape(metadata.get("title", "Untitled"))}</p>
<p>{escape(len(items))}</p>"""

    # -- Comments: body, trailing, indented --

    # Top-level comment

    yield "<div>"

    # Indented comment
    yield """<span>Text</span>"""  # Trailing comment

    yield "</div>"

