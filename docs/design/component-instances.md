# Component instances

## Create

Calling a component binds its props and returns a real `Component` instance. It does not execute the template body.

```python
greeting = Greeting(name="Ada")
```

## Render

```python
greeting.render()  # Render HTML to a string.
str(greeting)      # Same rendering behavior.
print(greeting)    # Uses str(greeting).
f"{greeting}"      # Renders too.
```

Every rendering operation executes the template again. There is no implicit output cache.

`repr(greeting)` does not render or execute template code.

Use explicit rendering at framework boundaries:

```python
return HTMLResponse(Greeting(name="Ada").render())
```

## Stream

```python
for chunk in greeting.render(stream=True):
    ...
```

Each call returns a fresh iterator and executes the template when iteration starts. Props belong to the instance, not the rendering method.

## Async

```python
html = await greeting.render()

async for chunk in greeting.render(stream=True):
    ...
```

String conversion of an async component raises a helpful error directing the caller to `await greeting.render()`. It never starts an event loop implicitly.

Sync and async templates return the same public `Component` type. The template determines whether rendering returns a direct value or an awaitable, and whether streaming returns an iterator or async iterator.

## Inspect

`Component.__wrapped__` references the generated Python function. Standard Python inspection follows it:

```python
inspect.signature(Greeting)
inspect.unwrap(Greeting)
inspect.getsource(Greeting)
```
