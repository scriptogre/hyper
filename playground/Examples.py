def Template(user: dict, items: list, count: int = 0, is_active: bool = True):
    _parts = []
    # Simple if
    def _attr(n, v):
        if v is True: return f' {n}'
        if v is False or v is None: return ''
        return f' {n}="{v}"'
    def _class(v):
        if isinstance(v, str): return v
        if isinstance(v, list): return ' '.join(filter(None, (_class(i) if isinstance(i, (list, dict)) else (str(i) if i else '') for i in v)))
        if isinstance(v, dict): return ' '.join(k for k, x in v.items() if x)
        return str(v) if v else ''
    def _style(v):
        if isinstance(v, str): return v
        if isinstance(v, dict): return ';'.join(f'{k}:{x}' for k, x in v.items())
        return str(v) if v else ''
    if is_active:
        _parts.append(f"""    <span>Active</span>""")

    # if/else
    if count > 0:
        _parts.append(f"""    <span>Has items</span>""")
    else:
        _parts.append(f"""    <span>No items</span>""")

    # if/elif/else chain
    if count == 0:
        _parts.append(f"""    <span class="empty">
        Chrispy
    </span>""")
    elif count == 1:
        _parts.append(f"""    <span class="single">
        One item
    </span>""")
    elif count < 10:
        _parts.append(f"""    <span class="few">
        A few items {count}
    </span>""")
    else:
        _parts.append(f"""    <span class="many">Many items</span>""")

    # Simple for loop
    _parts.append(f"""<ul>""")
    for item in items:
        _parts.append(f"""        <li>{item}</li>""")
    _parts.append(f"""</ul>
""")
    # Nested for loops
    _parts.append(f"""<table>""")
    for row in user['rows']:
        _parts.append(f"""        <tr>""")
        for cell in row:
            _parts.append(f"""                <td>{cell}</td>""")
        _parts.append(f"""        </tr>""")
    _parts.append(f"""</table>
""")
    # While loop
    while count < 10:
        _parts.append(f"""    <span>{count}</span>""")
        count = count + 1

    # Match/case
    match user['status']:
        case "active":
            _parts.append(f"""        <span class="green">Active</span>""")
        case "pending":
            _parts.append(f"""        <span class="yellow">Pending</span>""")
        case "inactive":
            _parts.append(f"""        <span class="red">Inactive</span>""")
        case _:
            _parts.append(f"""        <span class="gray">Unknown</span>""")

    # Function definition
    def greet(name: str):
        _parts = []
        _parts.append(f"""    <h1>Hello, {name}!</h1>""")
        return "".join(_parts)

    # Function with multiple statements
    def render_card(user: dict, show_details: bool):
        _parts = []
        _parts.append(f"""    <div class="card">
        <h2>{user['name']}</h2>""")
        if show_details:
            _parts.append(f"""            <p>{user['bio']}</p>""")
            for tag in user['tags']:
                _parts.append(f"""                <span class="tag">{tag}</span>""")
        _parts.append(f"""    </div>""")
        return "".join(_parts)

    # Deeply nested structure
    _parts.append(f"""<div class="container">""")
    if is_active:
        _parts.append(f"""        <div class="content">""")
        for section in user['sections']:
            _parts.append(f"""                <section>""")
            if section['visible']:
                match section['type']:
                    case "header":
                        _parts.append(f"""                                <h2>{section['title']}</h2>""")
                    case "paragraph":
                        _parts.append(f"""                                <p>{section['content']}</p>""")
                    case "list":
                        _parts.append(f"""                                <ul>""")
                        for item in section['items']:
                            _parts.append(f"""                                        <li>{item}</li>""")
                        _parts.append(f"""                                </ul>""")
            else:
                _parts.append(f"""                        <span class="hidden">Hidden section</span>""")
            _parts.append(f"""                </section>""")
        _parts.append(f"""        </div>""")
    _parts.append(f"""</div>
""")
    # Comments and Python code mixed with HTML
    total = 0
    for item in items:
        # Calculate running total
        total = total + item['price']
        _parts.append(f"""    <div class="item">
        <span class="name">{item['name']}</span>
        <span class="price">${item['price']}</span>
    </div>""")
    _parts.append(f"""<div class="total">Total: ${total}</div>
""")
    # ============================================
    # ADDITIONAL TEST SCENARIOS
    # ============================================

    # Multiple expressions on one line
    _parts.append(f"""<p>{user['first']} {user['last']} ({user['age']} years old)</p>
""")
    # Expressions in attributes
    _parts.append(f"""<div class="{user['theme']}" id="user-{user['id']}" data-count="{count}">
    <span style="color: {user['color']}">
        Styled text
    </span>
</div>
""")
    # Complex expressions
    _parts.append(f"""<span>{user['name'].upper()}</span>
<span>{", ".join(items)}</span>
<span>{len(items)} items</span>
<span>{items[0] if items else "empty"}</span>
""")
    # Nested dict access in expressions
    _parts.append(f"""<p>{user['address']['city']}, {user['address']['country']}</p>
""")
    # Method chaining
    _parts.append(f"""<p>{user['bio'].strip().capitalize()}</p>
""")
    # Arithmetic in expressions
    _parts.append(f"""<span>Total: {count * 10 + 5}</span>
<span>Average: {total / len(items) if items else 0}</span>
""")
    # Boolean expressions
    _parts.append(f"""<span>{is_active and "Yes" or "No"}</span>
<span>{"Active" if is_active else "Inactive"}</span>
""")
    # List/dict comprehension in expressions
    _parts.append(f"""<span>{[x * 2 for x in range(5)]}</span>
<span>{{ k: v.upper() for k, v in user.items() if isinstance(v, str) }}</span>
""")
    # Nested braces (dict literal inside expression)
    data = {"key": "value", "nested": {"a": 1}}
    _parts.append(f"""<span>{data}</span>
""")
    # F-string style with format specs
    _parts.append(f"""<span>{user['score']:.2f}</span>
<span>{count:03d}</span>
<span>{user['name']:>20}</span>
""")
    # Self-closing HTML tags
    _parts.append(f"""<img src="{user['avatar']}" alt="{user['name']}" />
<br />
<input type="text" value="{user['name']}" />
<hr />
""")
    # HTML with nested quotes (JSON needs escaped braces)
    _parts.append(f"""<div data-json='{{"name": "{user["name"]}"}}'></div>
<a href="/user/{user['id']}" title='View {user["name"]}'>Link</a>
""")
    # Empty control blocks (generates pass)
    if False:
        pass

    for x in []:
        pass

    # Consecutive HTML lines (no Python between)
    _parts.append(f"""<header>
    <nav>
        <ul>
            <li><a href="/">Home</a></li>
            <li><a href="/about">About</a></li>
            <li><a href="/contact">Contact</a></li>
        </ul>
    </nav>
</header>
""")
    # HTML with special characters and entities
    _parts.append(f"""<p>&lt;script&gt; is escaped</p>
<p>Copyright &copy; 2024</p>
<p>Price: &euro;{count}</p>
""")
    # Mixed indentation scenarios
    if is_active:
        _parts.append(f"""    <div>
        Level 1""")
        if count > 0:
            _parts.append(f"""            <div>
                Level 2""")
            for i in range(count):
                _parts.append(f"""                    <span>Level 3: {i}</span>""")
            _parts.append(f"""            </div>""")
        _parts.append(f"""    </div>""")

    # Inline conditionals in HTML context
    _parts.append(f"""<div class="card {'active' if is_active else 'inactive'} {'premium' if count > 100 else 'basic'}">
    Content
</div>
""")
    # Function with default parameter
    def badge(text: str, color: str = "blue"):
        _parts = []
        _parts.append(f"""    <span class="badge badge-{color}">{text}</span>""")
        return "".join(_parts)

    # Function returning after HTML
    def get_status_html(status: str):
        _parts = []
        match status:
            case "ok":
                _parts.append(f"""            <span class="green">OK</span>""")
            case "error":
                _parts.append(f"""            <span class="red">Error</span>""")
            case _:
                _parts.append(f"""            <span class="gray">Unknown</span>""")
        return "".join(_parts)

    # Class definition
    class Card:
        def __init__(self, title: str):
            self.title = title

        def render(self):
            _parts = []
            _parts.append(f"""        <div class="card">
            <h3>{self.title}</h3>
        </div>""")
            return "".join(_parts)

    # Lambda in expression
    sorter = lambda x: x['name']
    sorted_items = sorted(items, key=sorter)
    for item in sorted_items:
        _parts.append(f"""    <li>{item['name']}</li>""")

    # Walrus operator
    if (n := len(items)) > 0:
        _parts.append(f"""    <p>Found {n} items</p>""")

    # Unpacking in for loop
    pairs = [("a", 1), ("b", 2), ("c", 3)]
    for key, value in pairs:
        _parts.append(f"""    <dt>{key}</dt>
    <dd>{value}</dd>""")

    # Enumerate
    for i, item in enumerate(items):
        _parts.append(f"""    <li data-index="{i}">{item}</li>""")

    # Zip
    names = ["Alice", "Bob"]
    scores = [100, 95]
    for name, score in zip(names, scores):
        _parts.append(f"""    <tr><td>{name}</td><td>{score}</td></tr>""")

    # Slice expressions
    _parts.append(f"""<span>First three: {items[:3]}</span>
<span>Last two: {items[-2:]}</span>
<span>Every other: {items[::2]}</span>
""")
    # Complex nested match
    match user:
        case {"type": "admin", "level": level}:
            _parts.append(f"""        <span class="admin">Admin Level {level}</span>""")
        case {"type": "user", "verified": True}:
            _parts.append(f"""        <span class="verified">Verified User</span>""")
        case {"type": "user", "verified": False}:
            _parts.append(f"""        <span class="unverified">Unverified User</span>""")
        case _:
            _parts.append(f"""        <span>Guest</span>""")

    # Expression with getattr/hasattr
    _parts.append(f"""<span>{getattr(user, 'name', 'Anonymous')}</span>
""")
    # Multi-line HTML tag - put on single line (multi-line not supported)
    _parts.append(f"""<div class="container {user['theme']}" id="main-{user['id']}" data-active="{is_active}">
    <p>Multi-line tag content</p>
</div>
""")
    # Raw Python blocks between HTML
    _parts.append(f"""<section>""")
    result = []
    for i in range(5):
        result.append(i ** 2)
    squares = result
    _parts.append(f"""</section>
<ul>""")
    for sq in squares:
        _parts.append(f"""        <li>{sq}</li>""")
    _parts.append(f"""</ul>
""")
    # Comments interspersed
    _parts.append(f"""<div>""")
    # This is a comment inside HTML context
    _parts.append(f"""    <p>Paragraph 1</p>""")
    # Another comment
    _parts.append(f"""    <p>Paragraph 2</p>
</div>
""")
    # Escape sequences in strings
    message = "Hello\nWorld"
    _parts.append(f"""<pre>{message}</pre>
""")
    # Unicode in expressions
    _parts.append(f"""<span>{'★' * count}</span>
<span>{user.get('emoji', '👤')}</span>
""")
    # Chained comparisons
    if count > 0 and count < 100:
        _parts.append(f"""    <span>In range</span>""")

    # Not operator
    if not is_active:
        _parts.append(f"""    <span>Inactive</span>""")

    # In operator
    if "admin" in user.get('roles', []):
        _parts.append(f"""    <span class="admin-badge">Admin</span>""")

    # Try/except (if supported)
    try:
        _parts.append(f"""    <span>{risky_operation()}</span>""")
    except:
        _parts.append(f"""    <span>Error occurred</span>""")

    # Generator expression
    _parts.append(f"""<span>{sum(x for x in range(10))}</span>
<span>{",".join(str(x) for x in items)}</span>
""")
    # Nested ternary
    _parts.append(f"""<span>{"many" if count > 10 else "few" if count > 0 else "none"}</span>
""")
    # HTML inside string (edge case - should be treated as Python)
    html_string = "<div>Not actual HTML</div>"
    _parts.append(f"""<p>{html_string}</p>
""")
    # Very long expression
    _parts.append(f"""<p>{user['profile']['settings']['preferences']['theme']['primary_color'] if user.get('profile') and user['profile'].get('settings') else 'default'}</p>
""")
    # ============================================
    # CSS / STYLE SCENARIOS
    # ============================================

    # Inline styles with Python expressions
    _parts.append(f"""<div style="color: {user['color']}; font-size: {count}px;">Styled</div>
<div style="background: rgb({user['r']}, {user['g']}, {user['b']});">RGB</div>
""")
    # CSS custom properties
    _parts.append(f"""<div style="--primary-color: {user['theme_color']}; color: var(--primary-color);">Custom prop</div>
""")
    # CSS calc (braces in CSS don't interfere)
    _parts.append(f"""<div style="width: calc(100% - {count}px);">Calc</div>
""")
    # Style block (CSS braces should be preserved)
    _parts.append(f"""<style>
    .card {{ background: white; }}
    .card:hover {{ transform: scale(1.05); }}
    @media (max-width: 768px) {{
        .card {{ padding: 10px; }}
    }}
    @keyframes fade {{
        from {{ opacity: 0; }}
        to {{ opacity: 1; }}
    }}
</style>
""")
    # ============================================
    # JAVASCRIPT SCENARIOS
    # ============================================

    # Script block with JS (keep on single lines that start with < or use inline)
    _parts.append(f"""<script>const data = {{ name: "{user['name']}", count: {count} }}; console.log(data);</script>
""")
    # Or use type=module with src for external JS
    _parts.append(f"""<script type="module" src="/js/app.js" data-user-id="{user['id']}"></script>
""")
    # Inline event handlers
    _parts.append(f"""<button onclick="alert('Count: {count}')">Alert</button>
<button onclick="console.log({{ id: {user['id']} }})">Log</button>
""")
    # ============================================
    # ALPINE.JS SCENARIOS
    # ============================================

    # Alpine x-data with object literal (double braces to escape)
    _parts.append(f"""<div x-data="{{ open: false, count: {count} }}">
    <button @click="open = !open">Toggle</button>
    <div x-show="open">Content</div>
</div>
""")
    # Alpine x-for (looks like Python for, but it's Alpine)
    _parts.append(f"""<template x-for="item in items">
    <div x-text="item"></div>
</template>
""")
    # Alpine with Python data injected
    _parts.append(f"""<div x-data="{{ items: {items}, user: {user} }}">
    <template x-for="item in items">
        <span x-text="item.name"></span>
    </template>
</div>
""")
    # Alpine shorthand bindings
    _parts.append(f"""<div :class="{{ 'active': isActive, 'disabled': !isActive }}">Bound</div>
<input :value="count" @input="count = $event.target.value" />
""")
    # Alpine x-bind with Python expression
    _parts.append(f"""<div x-bind:class="count > 0 ? 'has-items' : 'empty'" data-count="{count}">
    Alpine conditional class
</div>
""")
    # ============================================
    # HTMX SCENARIOS
    # ============================================

    # HTMX attributes
    _parts.append(f"""<button hx-get="/api/items" hx-target="#list" hx-swap="innerHTML">Load</button>
<div hx-post="/api/items/{user['id']}" hx-trigger="click" hx-vals='{{"count": {count}}}'>
    HTMX with Python data
</div>
""")
    # HTMX with dynamic URLs
    _parts.append(f"""<a hx-get="/users/{user['id']}/profile" hx-push-url="true">View Profile</a>
""")
    # ============================================
    # SVG SCENARIOS
    # ============================================

    # SVG with Python expressions
    _parts.append(f"""<svg width="{count * 10}" height="100" viewBox="0 0 100 100">
    <circle cx="50" cy="50" r="{count}" fill="{user['color']}" />
    <text x="50" y="50" text-anchor="middle">{user['name']}</text>
    <path d="M 10 10 L {count} {count}" stroke="black" />
</svg>
""")
    # SVG with style
    _parts.append(f"""<svg>
    <style>
        .highlight {{ fill: yellow; }}
    </style>
    <rect class="highlight" width="{count}" height="50" />
</svg>
""")
    # ============================================
    # SPECIAL HTML SCENARIOS
    # ============================================

    # Pre/code blocks (content that looks like Python needs escaping or single-line)
    _parts.append(f"""<pre><code>function example() {{ return {count}; }}</code></pre>
""")
    # HTML comments (should be preserved)
    _parts.append(f"""<!-- User: {user['name']} -->
<div>
    <!-- TODO: Add more content -->
    Content here
</div>
""")
    # Data attributes with JSON
    _parts.append(f"""<div data-config='{{"id": {user["id"]}, "name": "{user["name"]}"}}'>Config</div>
<div data-items='{items}'>Items as JSON</div>
""")
    # Boolean attributes
    _parts.append(f"""<input type="checkbox" {"checked" if is_active else ""} />
<button {"disabled" if count == 0 else ""}>Submit</button>
""")
    # Custom elements / Web Components
    _parts.append(f"""<my-card user-id="{user['id']}" theme="{user['theme']}">
    <slot name="header">{user['name']}</slot>
</my-card>
""")
    # Template tag
    _parts.append(f"""<template id="card-template">
    <div class="card">
        <h3>{user['name']}</h3>
    </div>
</template>
""")
    # Iframe with srcdoc
    _parts.append(f"""<iframe srcdoc="<html><body><h1>{user['name']}</h1></body></html>"></iframe>
""")
    # ============================================
    # ESCAPING EDGE CASES
    # ============================================

    # Literal braces (doubled to escape)
    _parts.append(f"""<p>Use {{variable}} for templates</p>
<p>JSON example: {{"key": "value"}}</p>
""")
    # HTML entities for braces
    _parts.append(f"""<p>Opening brace: &#123; Closing: &#125;</p>
""")
    # Mixed escaped and unescaped
    _parts.append(f"""<p>Static {{braces}} and dynamic {count}</p>
""")
    # Triple braces edge case
    _parts.append(f"""<p>{{{user['name']}}}</p>
""")
    # ============================================
    # TAILWIND CSS SCENARIOS
    # ============================================

    # Long Tailwind class strings
    _parts.append(f"""<div class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 {user['extra_classes']}">
    <span class="text-lg font-semibold text-gray-900 dark:text-white">{user['name']}</span>
    <span class="px-2 py-1 text-sm {'bg-green-100 text-green-800' if is_active else 'bg-red-100 text-red-800'} rounded-full">
        {'Active' if is_active else 'Inactive'}
    </span>
</div>
""")
    # Tailwind with conditional classes
    _parts.append(f"""<div class="btn {'btn-primary' if user['role'] == 'admin' else 'btn-secondary'} {'btn-lg' if count > 10 else 'btn-sm'}">
    Button
</div>
""")
    # ============================================
    # EDGE CASE EXPRESSIONS
    # ============================================

    # Nested quotes in expressions
    _parts.append(f"""<div title='{user["name"]} said "hello"'>Quotes</div>
<div data-msg="{user['message'].replace('"', '&quot;')}">Escaped quotes</div>
""")
    # Backslashes (must use variable - backslash not allowed in f-string expr)
    path_normalized = user['path'].replace('\\', '/')
    _parts.append(f"""<div data-path="{path_normalized}">Path</div>
""")
    # Newlines in expressions (use chr() or variable)
    newline = "\n"
    _parts.append(f"""<pre>{newline.join(items)}</pre>
""")
    # Raw strings (backslash limitation - use variable)
    raw_text = r"\n is a newline"
    _parts.append(f"""<p>{raw_text}</p>
""")
    # Bytes (edge case)
    _parts.append(f"""<p>{user.get('data', b'').decode('utf-8')}</p>
""")
    # Complex dict/list literals mixed with expressions
    config = {"theme": user['theme'], "count": count, "items": [x for x in items if x]}
    _parts.append(f"""<div data-config="{config}">Complex config</div>
""")
    # Walrus in comprehension
    _parts.append(f"""<ul>""")
    for item in [y for x in items if (y := x.strip())]:
        _parts.append(f"""        <li>{item}</li>""")
    _parts.append(f"""</ul>
""")
    # ============================================
    # INTERNATIONALIZATION
    # ============================================

    # RTL text
    _parts.append(f"""<div dir="rtl" lang="ar">{user.get('arabic_name', 'مرحبا')}</div>
""")
    # Unicode normalization
    _parts.append(f"""<p>{user['name'].encode('utf-8').decode('utf-8')}</p>
""")
    # Emoji in content and attributes
    _parts.append(f"""<span title="{user.get('mood', '😊')}">{user.get('emoji', '👤')} {user['name']}</span>
""")
    # CJK characters
    _parts.append(f"""<p lang="ja">{user.get('japanese_name', 'こんにちは')}</p>
""")
    # ============================================
    # FORM SCENARIOS
    # ============================================

    # Form with dynamic action
    _parts.append(f"""<form action="/users/{user['id']}/update" method="POST">
    <input type="hidden" name="csrf" value="{user.get('csrf_token', '')}" />
    <input type="text" name="name" value="{user['name']}" placeholder="Name" />
    <select name="role">""")
    for role in ['user', 'admin', 'guest']:
        _parts.append(f"""            <option value="{role}" {"selected" if role == user.get('role') else ""}>{role.title()}</option>""")
    _parts.append(f"""    </select>
    <textarea name="bio">{user.get('bio', '')}</textarea>
    <button type="submit">Save</button>
</form>
""")
    # Input with datalist
    _parts.append(f"""<input list="suggestions" value="{user.get('search', '')}" />
<datalist id="suggestions">""")
    for suggestion in items[:5]:
        _parts.append(f"""        <option value="{suggestion}">""")
    _parts.append(f"""</datalist>
""")
    # ============================================
    # TABLE SCENARIOS
    # ============================================

    # Complex table with all features
    _parts.append(f"""<table class="w-full">
    <caption>{user['name']}'s Data ({len(items)} items)</caption>
    <thead>
        <tr>
            <th scope="col">#</th>
            <th scope="col">Name</th>
            <th scope="col">Value</th>
        </tr>
    </thead>
    <tbody>""")
    for i, item in enumerate(items):
        _parts.append(f"""            <tr class="{'bg-gray-100' if i % 2 == 0 else 'bg-white'}">
                <td>{i + 1}</td>
                <td>{item.get('name', 'N/A')}</td>
                <td>{item.get('value', 0):.2f}</td>
            </tr>""")
    _parts.append(f"""    </tbody>
    <tfoot>
        <tr>
            <td colspan="2">Total</td>
            <td>{sum(item.get('value', 0) for item in items):.2f}</td>
        </tr>
    </tfoot>
</table>
""")
    # ============================================
    # ACCESSIBILITY SCENARIOS
    # ============================================

    # ARIA attributes
    _parts.append(f"""<div role="alert" aria-live="polite" aria-label="{user['name']} notification">
    <span aria-hidden="true">⚠️</span>
    {user.get('alert_message', 'No alerts')}
</div>
""")
    # Skip links and landmarks
    _parts.append(f"""<a href="#main-content" class="sr-only focus:not-sr-only">Skip to content</a>
<main id="main-content" role="main" aria-labelledby="heading-{user['id']}">
    <h1 id="heading-{user['id']}">{user['name']}</h1>
</main>
""")
    # ============================================
    # ASSIGNING HTML TO VARIABLES
    # ============================================

    # Sub-component function returns HTML that can be stored
    def make_badge(text: str, color: str = "blue"):
        _parts = []
        _parts.append(f"""    <span class="badge badge-{color}">{text}</span>""")
        return "".join(_parts)

    # Call sub-component and use its result
    admin_badge = make_badge("Admin", "red")
    user_badge = make_badge("User", "green")

    _parts.append(f"""<div class="badges">
    {admin_badge}
    {user_badge}
    {make_badge("Guest", "gray")}
</div>
""")
    # Conditionally build HTML and assign to variable
    def render_status(status: str):
        _parts = []
        if status == "online":
            _parts.append(f"""        <span class="status online">●</span>""")
        elif status == "away":
            _parts.append(f"""        <span class="status away">◐</span>""")
        else:
            _parts.append(f"""        <span class="status offline">○</span>""")
        return "".join(_parts)

    status_html = render_status(user.get("status", "offline"))
    _parts.append(f"""<div class="user-status">{status_html}</div>
""")
    # Build a list of HTML fragments
    def render_tag(tag: str):
        _parts = []
        _parts.append(f"""    <span class="tag">{tag}</span>""")
        return "".join(_parts)

    tags_html = [render_tag(t) for t in user.get("tags", [])]
    _parts.append(f"""<div class="tag-list">""")
    for tag_html in tags_html:
        _parts.append(f"""        {tag_html}""")
    _parts.append(f"""</div>
""")
    # ============================================
    # NEW ATTRIBUTE FEATURES (compile-time)
    # ============================================

    # Boolean attributes - True renders, False omits
    _parts.append(f"""<input type="checkbox" {_attr('disabled', is_active)} />
<button {_attr('disabled', not is_active)}>Disabled when inactive</button>
<input type="text" {_attr('readonly', count == 0)} />
""")
    # Dynamic class with list and dict
    _parts.append(f"""<div class="{_class(["card", "shadow", {"active": is_active, "highlighted": count > 5}])}">
    Dynamic class
</div>
""")
    # Dynamic style with dict
    _parts.append(f"""<p style="{_style({"color": user.get("color", "black"), "font-size": f"{count + 12}px"})}">
    Dynamic style
</p>
""")
    # Combined - class list with conditional + boolean attr
    _parts.append(f"""<button
    class="{_class(["btn", {"btn-primary": user.get("role") == "admin", "btn-secondary": user.get("role") != "admin"}])}"
    {_attr('disabled', count == 0)}>
    Submit
</button>
""")
    # ============================================
    # CONTEXT MANAGERS (with statement)
    # ============================================

    # Simple with
    with open("/tmp/test.txt") as f:
        content = f.read()
        _parts.append(f"""    <pre>{content}</pre>""")

    # Nested with statements
    with user.get('lock', nullcontext()):
        _parts.append(f"""    <div class="critical-section">
        <span>Protected content</span>
    </div>""")

    # ============================================
    # TRY/EXCEPT PATTERNS
    # ============================================

    # Simple try/except
    try:
        _parts.append(f"""    <span>{user['missing_key']}</span>""")
    except KeyError:
        _parts.append(f"""    <span class="error">Key not found</span>""")

    # Multiple exception types
    try:
        value = int(user.get('number', 'invalid'))
        _parts.append(f"""    <span>Number: {value}</span>""")
    except ValueError as e:
        _parts.append(f"""    <span class="error">Invalid number: {e}</span>""")
    except TypeError as e:
        _parts.append(f"""    <span class="error">Type error: {e}</span>""")

    # try/except/else/finally
    try:
        result = user['required']
    except KeyError:
        _parts.append(f"""    <span class="warning">Using default</span>""")
        result = "default"
    else:
        _parts.append(f"""    <span class="info">Found value</span>""")
    finally:
        _parts.append(f"""    <span class="result"
          style="{_style(True)}"
          >""")
        Result: {result}
        _parts.append(f"""    </span>""")

    # ============================================
    # RECURSIVE FUNCTIONS
    # ============================================

    # Recursive tree rendering
    def render_tree(node: dict, depth: int = 0):
        _parts = []
        _parts.append(f"""    <div class="tree-node" style="margin-left: {depth * 20}px">
        <span class="label">{node.get('label', 'Node')}</span>""")
        if node.get('children'):
            _parts.append(f"""            <div class="children">""")
            for child in node['children']:
                render_tree(child, depth + 1)
            _parts.append(f"""            </div>""")
        _parts.append(f"""    </div>""")
        return "".join(_parts)

    # Call recursive function
    render_tree(user.get('tree', {'label': 'Root'}))

    # ============================================
    # FRAGMENT DECORATOR
    # ============================================

    _parts.append(f"""@fragment""")
    def TreeNode(n: dict, depth: int = 0):
        _parts = []
        _parts.append(f"""    <div class="tree-node" style="margin-left: {depth * 20}px">
        <span>{n.get('label', 'Node')}</span>
    </div>""")
        return "".join(_parts)

    # ============================================
    # SLOT PATTERN
    # ============================================

    # Slot placeholder for children content
    _parts.append(f"""<div class="card">
    <div class="card-header">
        <h2>{user['name']}</h2>
    </div>
    <div class="card-body">
        {...}
    </div>
</div>
""")
    # Final element
    _parts.append(f"""<footer>
    <p>Generated by Hyper</p>
</footer>""")
    return "".join(_parts)
