# Template Security

Secure templates by restricting capabilities, not syntax. Use the Gatekeeper Pattern to control the environment while allowing full Python language features.

## Import Firewall

Control access to modules by overriding the `__import__` function. Whitelist safe modules and block system access.

```python
import builtins

ALLOWED_MODULES = {
    "math", "datetime", "json",
    "app.models", "app.utils"
}

def secure_importer(name, globals=None, locals=None, fromlist=(), level=0):
    if name.split(".")[0] in ALLOWED_MODULES:
        return builtins.__import__(name, globals, locals, fromlist, level)
    raise SecurityError(f"Import denied: {name}")
```

## Runtime Audit Hooks

Use `sys.addaudithook` to intercept low-level interpreter events. This acts as a final failsafe against unauthorized OS interaction.

```python
import sys

def audit_hook(event, args):
    if is_rendering_template() and event in ["open", "os.system"]:
        raise SecurityError(f"Forbidden operation: {event}")

sys.addaudithook(audit_hook)
```

## Clean Room Scope

Isolate the execution context. Strip dangerous functions from builtins and separate global helpers from local props.

```python
safe_builtins = dict(builtins.__dict__)
for key in ['open', 'exec', 'eval']:
    del safe_builtins[key]
safe_builtins['__import__'] = secure_importer

def execute(code, props):
    globals_scope = {"__builtins__": safe_builtins, "User": app.User}
    exec(code, globals_scope, props.copy())
```

## Output Safety

Prevent XSS by auto-escaping all variable output. Require explicit opt-in for raw HTML.

- Default: `{content}` compiles to `yield escape(str(content))`
- Raw: `{content:safe}` compiles to `yield str(content)`
