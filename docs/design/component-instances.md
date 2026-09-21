# Component instances

**Status:** Agreed target. Not implemented. Supersedes the eager rendering API in the component language plan.

## Create

Calling a component binds its props and returns a real `Component` instance. It does not execute the template body.

```python
greeting = Greeting(name="Ada")
```

## Render

```python
greeting.render()  # Render HTML to a string.
str(greeting)     # Same rendering behavior.
print(greeting)   # Uses str(greeting).
f"{greeting}"     # Renders too.
```

Every rendering operation executes the template again. There is no implicit output cache.

`repr(greeting)` does not render or execute template code.

Use explicit rendering at framework boundaries:

```python
return HTMLResponse(Greeting(name="Ada").render())
```

## Stream

```python
for chunk in greeting.stream():
    ...
```

Each `.stream()` call returns a fresh iterator. Props belong to the instance, not the rendering method.

## Async

```python
html = await greeting.render()

async for chunk in greeting.stream():
    ...
```

String conversion of an async component raises a helpful error directing the caller to `await greeting.render()`. It never starts an event loop implicitly.
