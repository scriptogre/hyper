"""
Live search with debouncing and async support

Shows:
- Debounced events
- Async handlers
- Loading states
- Empty states
- Manual rendering with `await render()`
"""

import asyncio

# State
query = ""
results = []
loading = False

# Async handler
async def search_items(q: str):
    global query, results, loading

    query = q
    if not q:
        results = []
        return

    # Set loading state
    loading = True
    await render()  # Force render to show loading spinner

    # Simulate API call
    await asyncio.sleep(0.3)

    # Mock search results
    all_items = [
        "Apple", "Apricot", "Banana", "Blueberry",
        "Cherry", "Cranberry", "Date", "Dragonfruit",
        "Elderberry", "Fig", "Grape", "Guava"
    ]

    results = [
        item for item in all_items
        if q.lower() in item.lower()
    ]

    # Clear loading state
    loading = False
    # Framework auto-renders after handler completes

# Template
t"""
<div class="search-box">
    <input
        type="search"
        placeholder="Search fruits..."
        value="{query}"
        @input.debounce.300="search_items(value)"
    />

    {% if loading %}
    <div class="loading">
        <span class="spinner"></span>
        Searching...
    </div>
    {% elif query and not results %}
    <div class="empty">
        No results for "{query}"
    </div>
    {% elif results %}
    <ul class="results">
        {% for item in results %}
        <li>{item}</li>
        {% endfor %}
    </ul>
    {% endif %}
</div>
"""
