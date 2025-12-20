# Live Component API: V1 vs V2

Two approaches for adding live/stateful components to Hyper.

## V1: Function-Wrapped (with @live decorator)

```python
# app/components/counter.py
from hyper import live

@live
def counter():
    count = 0

    def increment():
        nonlocal count
        count += 1

    t"""
    <div>
        <p>Count: {count}</p>
        <button @click="increment">+</button>
    </div>
    """
```

### Pros
- ✅ Scoped state (`nonlocal` instead of `global`)
- ✅ Can return different components from same file
- ✅ Explicit opt-in (decorator is visible)

### Cons
- ❌ Different from regular Hyper components (wrapped in function)
- ❌ Requires decorator import
- ❌ Template not at top level
- ❌ Breaks Hyper's "type hints + t-string" pattern

---

## V2: Top-Level t-string (directory-based)

```python
# app/live/counter.py

count = 0

def increment():
    global count
    count += 1

t"""
<div>
    <p>Count: {count}</p>
    <button @click="increment">+</button>
</div>
"""
```

### Pros
- ✅ **Consistent with Hyper** - same structure as regular components
- ✅ **Zero boilerplate** - no decorator, no imports needed
- ✅ **Top-level t-string** - matches existing pattern
- ✅ **Clear separation** - directory indicates component type
- ✅ **Simpler mental model** - just move to `live/` directory

### Cons
- ⚠️ Requires `global` keyword (verbose)
- ⚠️ Directory convention might be "too magical"
- ⚠️ Can't have multiple live components in one file

---

## Side-by-Side: Todo List

### V1 (Function-Wrapped)

```python
from hyper import live

@live
def todos():
    todos = []
    next_id = 1

    def add_todo(text: str):
        nonlocal next_id
        todos.append({"id": next_id, "text": text, "done": False})
        next_id += 1

    def toggle(todo_id: int):
        for todo in todos:
            if todo["id"] == todo_id:
                todo["done"] = not todo["done"]
                break

    t"""
    <div>
        <ul>
            {% for todo in todos %}
            <li>
                <input
                    type="checkbox"
                    checked={todo.done}
                    @change="toggle({todo.id})"
                />
                {todo.text}
            </li>
            {% endfor %}
        </ul>
        <form @submit.prevent="add_todo(text)">
            <input name="text" />
        </form>
    </div>
    """
```

**Lines:** 30
**Boilerplate:** `from hyper import live` + `@live` + `def todos():`

---

### V2 (Top-Level)

```python
todos = []
next_id = 1

def add_todo(text: str):
    global next_id
    todos.append({"id": next_id, "text": text, "done": False})
    next_id += 1

def toggle(todo_id: int):
    for todo in todos:
        if todo["id"] == todo_id:
            todo["done"] = not todo["done"]
            break

t"""
<div>
    <ul>
        {% for todo in todos %}
        <li>
            <input
                type="checkbox"
                checked={todo.done}
                @change="toggle({todo.id})"
            />
            {todo.text}
        </li>
        {% endfor %}
    </ul>
    <form @submit.prevent="add_todo(text)">
        <input name="text" />
    </form>
</div>
"""
```

**Lines:** 27
**Boilerplate:** Just `global` keyword

---

## Comparison with Regular Components

### Regular Hyper Component

```python
# app/components/button.py

variant: str = "primary"
disabled: bool = False

t"""
<button class="btn btn-{variant}" disabled={disabled}>
    {...}
</button>
"""
```

### V1 Live Component

```python
# app/components/button.py
from hyper import live

@live
def button():
    variant: str = "primary"
    disabled: bool = False
    clicked = False

    def click():
        nonlocal clicked
        clicked = True

    t"""
    <button
        class="btn btn-{variant}"
        disabled={disabled}
        @click="click"
    >
        {...}
    </button>
    """
```

**Structure:** Different (wrapped in function)

---

### V2 Live Component

```python
# app/live/button.py

variant: str = "primary"
disabled: bool = False
clicked = False

def click():
    global clicked
    clicked = True

t"""
<button
    class="btn btn-{variant}"
    disabled={disabled}
    @click="click"
>
    {...}
</button>
"""
```

**Structure:** Same! Just in different directory.

---

## The `global` Keyword Issue

V2's main drawback is requiring `global` to mutate state:

```python
count = 0

def increment():
    global count  # Required
    count += 1
```

### Alternatives to Explore

#### Option A: Keep `global` (standard Python)
**Verdict:** Most explicit, works with all tooling

#### Option B: Use a `state` object
```python
state = {
    "count": 0
}

def increment():
    state["count"] += 1  # No global needed
```
**Verdict:** More verbose, loses type hints

#### Option C: Framework magic (auto-detect mutations)
```python
count = 0

def increment():
    count += 1  # Framework captures this somehow
```
**Verdict:** Too magical, confuses linters

#### Option D: Use closures (wrap module)
```python
# Framework wraps module in function internally
def _component():
    count = 0
    def increment():
        nonlocal count  # Works!
        count += 1
    t"""..."""
```
**Verdict:** Tricky to implement, might break imports

---

## Recommendation: **V2 (Top-Level)**

Why?
1. **Consistency is king** - matching regular components is critical
2. **Zero boilerplate** - no imports or decorators
3. **Clear mental model** - directory = component type
4. **Top-level t-string** - feels like Hyper

The `global` keyword is a small price to pay for consistency.

---

## Implementation Note

V2 could even support an **optional decorator** for components outside `live/`:

```python
# app/components/button.py
from hyper import live

live = True  # Or: __live__ = True

count = 0

def increment():
    global count
    count += 1

t"""..."""
```

**Directory = convention, marker = explicit opt-in.**

Best of both worlds!

---

## Vote

Which do you prefer?

- **V1:** Function-wrapped with `@live` decorator
- **V2:** Top-level t-string with directory convention

I vote **V2** for consistency with Hyper's design DNA.
