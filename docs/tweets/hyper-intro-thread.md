# Hyper Introduction Thread

---

## Tweet 1

Building templates for hypermedia-driven Python apps.

Not a new language. It's Python. You just write `end` instead of relying on indentation—plays nicer with HTML's block structure.

100% Python. 100% HTML. 100% editor support.

```hyper
users: list[User]

<ul>
    for user in users:
        <li>
            {user.name}
            if user.is_admin:
                <span class="badge">Admin</span>
            end
        </li>
    end
</ul>
```

---

## Tweet 2

Each .hyper file is a function. Top-level type hints are parameters.

```
# Card.hyper

title: str = "Untitled"
color: str = "blue"

<div class="card-{color}">
    <h3>{title}</h3>
    {...}
</div>
```

Transpiles to:

```python
def Card(title: str = "Untitled", color: str = "blue"):
    ...
```

---

## Tweet 3

Slots, named slots, attribute spreading:

```
# Layout.hyper

<body>
    <aside>{... name="sidebar"}</aside>
    <main>{...}</main>
</body>

# Button.hyper

<button class="btn" {...}>
    {...}
</button>
```

`{...}` spreads attributes or slots content based on context.

---

## Tweet 4

Inspired by templ (Go), Astro (JS), and tdom (Python). Bringing these ideas to Python.

Works with Django, FastAPI, Starlette. Your framework handles routing. Hyper handles templates.

JetBrains plugin works. VSCode next.

github.com/scriptogre/hyper
