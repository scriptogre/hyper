"""
Dropdown with pure client-side state

This demonstrates that NOT everything needs server state.
The dropdown expanded/collapsed state is purely client-side.

NO server state. NO WebSocket. Just _hyperscript.
"""

# Props (from parent)
items: list[str]
label: str = "Select..."

# Template
t"""
<div class="dropdown">
    <button
        class="dropdown-toggle"
        _="on click toggle .open on closest .dropdown"
    >
        {label} ▾
    </button>

    <ul class="dropdown-menu">
        {% for item in items %}
        <li
            _="on click
                put my innerText into the previous .dropdown-toggle then
                remove .open from closest .dropdown"
        >
            {item}
        </li>
        {% endfor %}
    </ul>
</div>

<style>
.dropdown {
    position: relative;
    display: inline-block;
}

.dropdown-toggle {
    padding: 8px 16px;
    border: 1px solid #ddd;
    background: white;
    cursor: pointer;
}

.dropdown-menu {
    display: none;
    position: absolute;
    top: 100%;
    left: 0;
    background: white;
    border: 1px solid #ddd;
    list-style: none;
    margin: 4px 0 0 0;
    padding: 0;
    min-width: 100%;
    z-index: 1000;
}

.dropdown.open .dropdown-menu {
    display: block;
}

.dropdown-menu li {
    padding: 8px 16px;
    cursor: pointer;
}

.dropdown-menu li:hover {
    background: #f5f5f5;
}
</style>
"""
