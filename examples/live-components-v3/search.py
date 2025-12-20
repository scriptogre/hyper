"""
Live search with debouncing

Shows:
- Async server handlers
- Debounced events with _hyperscript
- Loading states (client-side)
- Server-side data fetching
"""

import asyncio

# Server state
query = ""
results = []
loading = False

# Async handler
async def search(q: str):
    global query, results, loading

    query = q
    if not q:
        results = []
        loading = False
        return

    loading = True

    # Simulate API call
    await asyncio.sleep(0.3)

    # Mock results
    all_items = [
        "Apple", "Apricot", "Banana", "Blueberry",
        "Cherry", "Cranberry", "Date", "Dragonfruit",
        "Elderberry", "Fig", "Grape", "Guava"
    ]

    results = [item for item in all_items if q.lower() in item.lower()]
    loading = False

# Template
t"""
<div class="search-box">
    <div class="input-wrapper">
        <input
            type="search"
            placeholder="Search fruits..."
            value="{query}"
            _="on keyup debounced at 300ms
                add .loading to .input-wrapper then
                {search(value)}"
        />

        <span class="spinner {'visible' if loading else ''}"></span>
    </div>

    {% if not loading and query and not results %}
    <div class="empty">
        No results for "{query}"
    </div>
    {% elif results %}
    <ul class="results" _="on load show me with *fade-in">
        {% for item in results %}
        <li>{item}</li>
        {% endfor %}
    </ul>
    {% endif %}
</div>

<style>
.input-wrapper {
    position: relative;
}

.spinner {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    width: 16px;
    height: 16px;
    border: 2px solid #ddd;
    border-top-color: #333;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    opacity: 0;
    transition: opacity 0.2s;
}

.spinner.visible {
    opacity: 1;
}

@keyframes spin {
    to { transform: translateY(-50%) rotate(360deg); }
}

.results {
    animation: fadeIn 200ms ease;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
}
</style>
"""
