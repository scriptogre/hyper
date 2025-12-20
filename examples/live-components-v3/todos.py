"""
Todo list with mixed client/server state

Shows:
- Server state (todos list - must persist)
- Client state (UI animations - no need for server)
- Form handling with `{}`
- Client-side DOM manipulation with _hyperscript
"""

from typing import TypedDict

class Todo(TypedDict):
    id: int
    text: str
    done: bool

# Server state
todos: list[Todo] = []
next_id = 1

# Server handlers
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

# Computed
remaining = len([t for t in todos if not t["done"]])

# Template
t"""
<div class="todo-app">
    <h1>Tasks <small>({remaining} remaining)</small></h1>

    <form>
        <input name="text" placeholder="What needs to be done?" />

        <button _="
            on click
                {add_todo(text)} then
                set the previous <input/>'s value to ''
        ">
            Add
        </button>
    </form>

    <ul class="todo-list">
        {% for todo in todos %}
        <li>
            <!-- Server: update done state -->
            <input
                type="checkbox"
                checked={todo.done}
                _="on change {toggle({todo.id})}"
            />

            <!-- Client: instant visual feedback -->
            <span
                class="{'done' if todo.done else ''}"
                _="on change from previous <input/>
                    toggle .done on me"
            >
                {todo.text}
            </span>

            <!-- Mixed: fade out, then delete from server -->
            <button
                class="delete"
                _="on click
                    add .fade-out to closest <li/> then
                    wait 200ms then
                    {delete({todo.id})}"
            >×</button>
        </li>
        {% endfor %}
    </ul>

    {% if todos %}
    <footer>
        <button _="on click {clear_completed}">
            Clear completed
        </button>
    </footer>
    {% endif %}
</div>

<style>
.todo-list .done {
    text-decoration: line-through;
    opacity: 0.6;
}

.fade-out {
    animation: fadeOut 200ms ease;
}

@keyframes fadeOut {
    to { opacity: 0; transform: translateX(-10px); }
}
</style>
"""
