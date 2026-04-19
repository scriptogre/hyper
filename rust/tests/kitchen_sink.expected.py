from collections.abc import Iterable
from hyper import html, escape, safe, render_class, render_style, render_attr, render_data, render_aria, spread_attrs


@html
def KitchenSink(_content: Iterable[str] | None = None, *, name: str, count: int = 0, is_active: bool = True, items: list = [], metadata: dict = {}, callback: object = None, style: str = "default", raw_html: str = "", value: int = 0, variant: str = "primary", limit: int = 10, sections: list = [], pairs: list = [], names: list = [], scores: list = [], matrix: list = [], _header: Iterable[str] | None = None, _sidebar: Iterable[str] | None = None):
    # Kitchen sink: exercises every syntax construct for visual IDE smoke testing.
    # Open this file in JetBrains after any injection change and verify highlighting.
    ########################################
    # STATEMENTS
    ########################################

    result = name.upper()
    values = [x * 2 for x in range(count)]
    lookup = {k: v for k, v in metadata.items()}

    ########################################
    # ELEMENTS
    ########################################
    yield f"""\
<div class="container" id="main-{escape(count)}" data-name="{escape(name)}">
    <h1>{escape(name)}</h1>
    <p>Count is {escape(count + 1)} and active is {escape(is_active)}</p>
    <span>{escape(f"Hello, {name}!")}</span>
</div>"""

    ########################################
    # SELF-CLOSING
    ########################################
    yield f"""\
<div>
    <img src="/img/{escape(name)}.png" alt="{escape(name)}" />
    <input type="text" value="{escape(name)}"{render_attr("disabled", is_active)} />
    <br />
    <hr />
    <meta charset="utf-8" />
    <link rel="stylesheet" href="style.css" />
</div>
<div />
<span />"""

    ########################################
    # EMPTY ELEMENTS
    ########################################
    yield """\
<div></div>
<span></span>
<p></p>"""

    ########################################
    # DEEP NESTING
    ########################################
    yield """\
<div class="level-1">
    <div class="level-2">
        <div class="level-3">
            <div class="level-4">
                <div class="level-5">
                    <span>Deep content</span>
                </div>
            </div>
        </div>
    </div>
</div>"""

    ########################################
    # TEXT POSITIONS
    ########################################
    yield """\
Text before any element
<div>Inside div</div>
Text between elements
<span>Inside span</span>
Text after elements"""

    ########################################
    # ATTRIBUTES
    ########################################
    yield f"""\
<div class="static" id="s-{escape(count)}" data-val="{escape(name)}"{render_attr("is_active", is_active)}{render_attr("metadata", metadata)}>
    Mixed attributes
</div>"""
    # Shorthand
    yield f"""\
<input{render_attr("name", name)}{render_attr("value", value)}{render_attr("is_active", is_active)} />
<button{render_attr("name", name)}{render_attr("variant", variant)}{render_attr("is_active", is_active)}>Click</button>
<input type="text"{render_attr("name", name)} class="input" />"""
    # Spread on HTML
    yield f"""\
<a{spread_attrs(metadata)}>External link</a>
<div id="main"{spread_attrs(metadata)} class="container">Content</div>"""
    # Nested braces in attributes
    yield f"""\
<div class="{render_class(["card", {"sale": is_active}])}">Product</div>
<div style="{render_style({"color": "red", "font": {"size": "12px"}})}">Styled</div>"""

    ########################################
    # TEMPLATE ATTRIBUTES
    ########################################
    yield f"""\
<button class="btn btn-{escape(variant)}">Click</button>
<div data-info="{escape(count)}-{escape(variant)}">Info</div>
<a href="/users/{escape(count)}" class="link link-{escape(variant)}">Profile</a>
<span data-key="{escape(count)}{escape(variant)}">Adjacent expressions</span>"""

    ########################################
    # CLASS LIST AND STYLE DICT
    ########################################

    class_ = ["btn", "btn-primary", {"active": is_active}]
    yield f"""<button{render_attr("class_", class_)}>Class list</button>"""
    styles = {"color": "red", "font-weight": "bold"}
    yield f"""<p{render_attr("styles", styles)}>Styled</p>"""

    ########################################
    # DATA AND ARIA
    ########################################

    data = {"user-id": 123, "role": "admin"}
    aria = {"label": "Close", "hidden": is_active}
    yield f"""<div{render_data(data)}{render_aria(aria)}>Data and aria</div>"""

    ########################################
    # RESERVED KEYWORDS
    ########################################

    type_ = "button"
    yield f"""<button class="{render_class(class_)}" type="{escape(type_)}">Reserved keywords</button>"""

    ########################################
    # BOOLEAN ATTRIBUTES
    ########################################
    yield f"""\
<button{render_attr("disabled", is_active)}>Submit</button>
<input type="checkbox" checked />
<input type="checkbox" disabled />"""

    ########################################
    # SAFE HELPER
    ########################################
    yield f"""\
{escape(safe(raw_html))}
<div>{escape(safe("<b>bold</b>"))}</div>"""

    ########################################
    # IF / ELIF / ELSE
    ########################################

    if is_active:
        yield """<span class="active">Active</span>"""
    elif count > 0:
        yield f"""<span class="partial">Partial ({escape(count)})</span>"""
    else:
        yield """<span class="inactive">Inactive</span>"""

    ########################################
    # FOR LOOP
    ########################################

    for item in items:
        yield f"""<li class="item">{escape(item)}</li>"""

    ########################################
    # FOR DESTRUCTURING
    ########################################

    for key, val in metadata.items():
        yield f"""\
<dt>{escape(key)}</dt>
    <dd>{escape(val)}</dd>"""

    ########################################
    # WHILE LOOP
    ########################################

    while count > 0:
        yield f"""\
<p>Counting down: {escape(count)}</p>
    """
        count = count - 1

    ########################################
    # MATCH / CASE
    ########################################

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

    ########################################
    # TRY / EXCEPT / ELSE / FINALLY
    ########################################

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

    ########################################
    # WITH
    ########################################

    with open("/dev/null") as f:
        yield f"""<pre>{escape(f.read())}</pre>"""

    ########################################
    # DEFINITIONS
    ########################################

    @fragment
    def Badge(text: str, badge_variant: str = "info"):
        yield f"""<span class="badge badge-{escape(badge_variant)}">{escape(text)}</span>"""
    @cache
    @fragment
    def CachedList(entries: list):
        yield """<ul>"""

        for entry in entries:
            yield f"""\
<li>{escape(entry)}</li>
        """
        yield """</ul>"""

    @fragment(name="card")
    def Card(title: str):
        yield """<div class="card">"""
        yield f"""\
<h2>{escape(title)}</h2>
        """
        if _content is not None:
            yield from _content

        yield """</div>"""

    def format_name(n: str) -> str:
        return n.upper()
    yield f"""\
{escape(Badge("New"))}
{escape(CachedList(items))}"""

    ########################################
    # COMPONENTS
    ########################################

    yield from Badge(text="Sale", badge_variant="danger")

    yield from Badge()

    yield from Badge(is_active=is_active)

    yield from Badge(text=format_name(name))

    # <{CachedList}>
    def _cached_list():
        yield """<p>Fallback content</p>"""
    yield from CachedList(_cached_list(), entries=items)
    # </{CachedList}>

    yield from callback()

    ########################################
    # SLOTS
    ########################################

    if _header is not None:
        yield from _header
    else:
        yield """<h2>Default Header</h2>"""

    if _sidebar is not None:
        yield from _sidebar
    else:
        yield """<nav>Default Nav</nav>"""

    if _content is not None:
        yield from _content

    ########################################
    # NESTED CONTROL FLOW
    ########################################

    yield """<section>"""

    if is_active:
        for item in items:
            yield from Badge(text=item)

            yield """<div class="wrapper">"""

            match item:
                case "special":
                    yield from CachedList(entries=[item, item])

                case _:
                    yield f"""\
<span>{escape(item)}</span>
                """
            yield """</div>"""

    yield """</section>"""

    ########################################
    # BREAK / CONTINUE
    ########################################

    yield """<ul>"""

    for item in items:
        if item == "stop":
            break
        yield f"""\
<li>{escape(item)}</li>
    """
    yield """</ul>"""

    yield """<ul>"""

    for item in items:
        if item.startswith("_"):
            continue
        yield f"""\
<li>{escape(item)}</li>
    """
    yield """</ul>"""

    c = 0
    while True:
        if c >= limit:
            break
        yield f"""\
<span>{escape(c)}</span>
    """
        c = c + 1

    ########################################
    # LOOP PATTERNS
    ########################################

    for i, item in enumerate(items):
        yield f"""<li data-index="{escape(i)}">{escape(item)}</li>"""
    for num, item in enumerate(items, start=1):
        yield f"""<li value="{escape(num)}">{escape(item)}</li>"""
    for key, val in pairs:
        yield f"""\
<dt>{escape(key)}</dt>
    <dd>{escape(val)}</dd>"""
    for n, score in zip(names, scores):
        yield f"""<td>{escape(n)}</td><td>{escape(score)}</td>"""
    for row in matrix:
        yield """<tr>"""

        for cell in row:
            yield f"""\
<td>{escape(cell)}</td>
        """
        yield """</tr>"""

    for i in range(5):
        yield f"""<li>Item {escape(i)}</li>"""
    for item in reversed(items):
        yield f"""<li>{escape(item)}</li>"""

    ########################################
    # MATCH GUARDS
    ########################################

    match value:
        case x if x < 0:
            yield f"""\
<span>Negative: {escape(x)}</span>
    """
        case x if x == 0:
            yield """\
<span>Zero</span>
    """
        case x if x > 100:
            yield f"""\
<span>Large: {escape(x)}</span>
    """
        case x:
            yield f"""<span>Normal: {escape(x)}</span>"""
    match metadata:
        case {"type": "user", "admin": True}:
            yield """\
<span>Admin user</span>
    """
        case {"type": "user", "admin": False}:
            yield """\
<span>Regular user</span>
    """
        case {"type": t} if t.startswith("system"):
            yield f"""\
<span>System: {escape(t)}</span>
    """
        case _:
            yield """<span>Unknown</span>"""

    ########################################
    # TRY VARIANTS
    ########################################

    try:
        yield f"""<span>{escape(callback())}</span>"""
    except ValueError as e:
        yield f"""<span>Value error: {escape(e)}</span>"""
    except TypeError as e:
        yield f"""<span>Type error: {escape(e)}</span>"""
    except Exception as e:
        yield f"""<span>Unknown error: {escape(e)}</span>"""
    try:
        result = metadata['key']
    except KeyError:
        yield """<span>Missing</span>"""
    else:
        yield f"""<span>Found: {escape(result)}</span>"""
    try:
        yield f"""<span>{escape(metadata['value'])}</span>"""
    except:
        yield """<span>Error</span>"""
    finally:
        yield """<span>Cleanup</span>"""

    ########################################
    # EMPTY BLOCKS
    ########################################

    if is_active:
        pass
    if is_active:
        yield """<span>Yes</span>"""
    else:
        pass
    for item in items:
        pass
    match is_active:
        case True:
            pass
        case False:
            yield """<span>False</span>"""
    def empty():
        pass

    ########################################
    # COMPLEX NESTING
    ########################################

    yield """<div class="container">"""

    if is_active:
        for section in sections:
            yield f"""<section id="section-{escape(section['id'])}">"""

            match section['type']:
                case "header":
                    yield f"""\
<h1>{escape(section['title'])}</h1>
                    """
                case "list":
                    yield """<ul>"""

                    for item in section['items']:
                        if item.get('active'):
                            yield f"""\
<li class="active">{escape(item['name'])}</li>
                                """
                        else:
                            yield f"""\
<li>{escape(item['name'])}</li>
                                """
                    yield """</ul>"""

                case _:
                    yield """\
<div>Unknown type</div>
                """
            yield """</section>"""

    else:
        yield """\
<p>Content hidden</p>
    """
    yield """</div>"""

    ########################################
    # EXPRESSIONS
    ########################################
    yield f"""\
<span>{escape(metadata['key'])}</span>
<span>{escape(metadata.get('key', 'default'))}</span>
<span>{escape(name.strip().upper())}</span>
<span>{escape(', '.join(items))}</span>
<span>{escape('yes' if count > 0 else 'no')}</span>
<span>{escape(count * 2 + 1)}</span>
<span>{escape(items[0])}</span>
<span>{escape(items[1:3])}</span>
<span>{escape(len(items))}</span>
<span>{count:03d}</span>
<span>{3.14159:.2f}</span>"""

    ########################################
    # COMPREHENSIONS
    ########################################
    yield f"""\
<span>{escape([x * 2 for x in range(5)])}</span>
<span>{escape([item.upper() for item in items if item])}</span>
<span>{{k: v for k, v in metadata.items()}}</span>
<span>{{x for x in items}}</span>
<span>{escape(sum(x for x in range(10)))}</span>"""

    ########################################
    # ADJACENT EXPRESSIONS
    ########################################
    yield f"""\
<p>{escape(name)}{escape(count)}{escape(is_active)}</p>
<span>{escape(count)} {escape("item" if count == 1 else "items")}</span>"""

    ########################################
    # ESCAPED BRACES
    ########################################
    yield f"""\
<code>Use {{braces}} in templates</code>
<p>JSON: {{"key": "value"}}</p>
<p>Static {{braces}} and dynamic {escape(name)}</p>
<p>{{{{nested}}}}</p>"""

    ########################################
    # NESTED ELEMENTS
    ########################################
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

    ########################################
    # KEYWORDS AS CONTENT
    ########################################
    yield """\
<p>for example, this is just text</p>
<div>if you think about it, this makes sense</div>
<span>while we wait, have some tea</span>
<article>try our new product today</article>
<p>match the following pairs</p>
<blockquote>with great power comes great responsibility</blockquote>
<p>def initely the best approach</p>"""

    ########################################
    # PYTHON FEATURES
    ########################################

    if (n := len(items)) > 0:
        yield f"""<span>Found {escape(n)} items</span>"""
    sorter = lambda x: x.lower()
    first, *rest = items
    merged = {**metadata, "extra": "value"}
    yield f"""\
<span>{escape("positive" if value > 0 else "zero" if value == 0 else "negative")}</span>
<span>{value=}</span>
<span>{items!r}</span>"""

    ########################################
    # UNICODE
    ########################################
    yield """\
<span>Hello 👋</span>
<span>Stars: ⭐⭐⭐</span>
<p lang="zh">你好世界</p>
<p lang="ja">こんにちは</p>
<p dir="rtl" lang="ar">مرحبا بالعالم</p>
<span>Arrows: ← → ↑ ↓</span>
<span>Math: ∞ ≠ ≤ ≥ ± × ÷</span>"""

    ########################################
    # HTML ENTITIES
    ########################################
    yield f"""\
<p>&lt;script&gt; is escaped</p>
<p>Copyright: &copy; 2024</p>
<p>&#60;angle brackets&#62;</p>
<p>&copy; {escape(2024)} All rights reserved</p>"""

    ########################################
    # COMMENTS
    ########################################

    # Top-level comment
    yield """<div>"""

    # Indented comment
    yield """<span>Text</span>"""  # Trailing comment
    yield """</div>"""
    yield f"""\
<th scope="col">#</th>
<a href="#section">Jump</a>
<div style="color: #ff0000">Red</div>
<span>{escape(name or '#')}</span>
<p>Text # with hash # multiple # times</p>"""

    ########################################
    # WHITESPACE
    ########################################
    yield """\
<div>After blank lines</div>
<span>Between blank lines</span>"""
