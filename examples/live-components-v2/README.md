# Live Components V2: Top-Level t-string Pattern

This directory contains examples using the **top-level t-string pattern** - matching Hyper's existing component structure.

## Core Pattern

```python
# app/live/component.py

# State: module-level variables
count = 0

# Handlers: module-level functions
def increment():
    global count
    count += 1

# Template: top-level t-string
t"""
<div>
    <p>{count}</p>
    <button @click="increment">+</button>
</div>
"""
```

## Key Differences from V1

### V1 (Function-wrapped)
```python
@live
def counter():
    count = 0
    def increment():
        nonlocal count
        count += 1
    t"""..."""
```

### V2 (Top-level)
```python
# Just put in app/live/ directory

count = 0

def increment():
    global count
    count += 1

t"""..."""
```

## Why V2 is Better

1. **Consistent with Hyper** - Same structure as regular components
2. **Zero boilerplate** - No decorator, no function wrapper
3. **Standard Python** - Just modules, variables, functions
4. **Clear separation** - Directory = component type

## Examples

- **[counter.py](./counter.py)** - Minimal counter (7 lines!)
- **[todos.py](./todos.py)** - Todo list with validation
- **[search.py](./search.py)** - Async search with debouncing
- **[chat.py](./chat.py)** - Multi-user chat with shared state
- **[form_validation.py](./form_validation.py)** - Live form validation

## Comparison

### Regular Component
```python
# app/components/card.py

title: str
color: str = "blue"

t"""
<div class="card card-{color}">
    <h3>{title}</h3>
    {...}
</div>
"""
```

### Live Component
```python
# app/live/card.py

# Props (from parent)
title: str
color: str = "blue"

# State (mutable)
expanded = False

def toggle():
    global expanded
    expanded = not expanded

t"""
<div class="card card-{color}">
    <h3 @click="toggle">{title}</h3>
    {% if expanded %}
    <div class="content">{...}</div>
    {% endif %}
</div>
"""
```

**The only difference:** Directory location and event bindings!

## State Management

### Per-Connection (Default)
```python
count = 0  # Each user gets their own
```

### Shared
```python
from hyper import shared

messages = shared([])  # All users share
```

### Props
```python
user_id: int  # From parent (read-only)
```

## Event Binding

```python
@click="handler"                    # Basic
@submit.prevent="save"              # Prevent default
@input.debounce.300="search(value)" # Debounced
@click="delete(123)"                # With arguments
```

## Lifecycle Hooks

```python
def on_mount():
    """Called when user connects"""
    pass

def on_unmount():
    """Called when user disconnects"""
    pass

def on_error(error: Exception):
    """Called when handler raises"""
    pass
```

## Type Safety

```python
def save(name: str, age: int, admin: bool):
    # Automatic validation and conversion
    # name: must be string
    # age: converted from "25" to 25
    # admin: converted from "true" to True
    pass
```

## The Complete API

**Everything you need to know:**

1. Put component in `app/live/` directory
2. Module-level variables = state
3. Module-level functions = handlers
4. Use `global` to mutate state
5. Top-level t-string = template
6. `@event` attributes = bindings

**That's it!** 6 concepts. Pure Python. Zero magic.

---

See [docs/live-state-proposal-v2.md](../../docs/live-state-proposal-v2.md) for full proposal.
