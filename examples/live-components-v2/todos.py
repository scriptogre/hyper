"""
Live todo list with validation and type safety

Shows:
- List state management
- Form handling with type-safe parameters
- Computed values (just Python!)
- Event binding with arguments
"""

from typing import TypedDict

class Todo(TypedDict):
    id: int
    text: str
    done: bool

# State
todos: list[Todo] = []
next_id = 1

# Handlers
def add_todo(text: str):
    global next_id
    if not text.strip():
        return

    todos.append({
        "id": next_id,
        "text": text.strip(),
        "done": False
    })
    next_id += 1

def toggle(todo_id: int):
    for todo in todos:
        if todo["id"] == todo_id:
            todo["done"] = not todo["done"]
            break

def delete(todo_id: int):
    global todos
    todos = [t for t in todos if t["id"] != todo_id]

def clear_completed():
    global todos
    todos = [t for t in todos if not t["done"]]

# Computed values (just Python!)
remaining = len([t for t in todos if not t["done"]])

# Template
t"""
<div class="todo-app">
    <h1>Tasks <small>({remaining} remaining)</small></h1>

    <form @submit.prevent="add_todo(text)">
        <input
            name="text"
            placeholder="What needs to be done?"
            autofocus
        />
        <button type="submit">Add</button>
    </form>

    <ul class="todo-list">
        {% for todo in todos %}
        <li class="{'done' if todo.done else ''}">
            <input
                type="checkbox"
                checked={todo.done}
                @change="toggle({todo.id})"
            />
            <span>{todo.text}</span>
            <button
                class="delete"
                @click="delete({todo.id})"
            >×</button>
        </li>
        {% endfor %}
    </ul>

    {% if todos %}
    <footer>
        <button @click="clear_completed">
            Clear completed
        </button>
    </footer>
    {% endif %}
</div>
"""
