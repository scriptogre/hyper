# Configuration

**Status:** Idea, not yet implemented. Deferred until the zero-config-imports work lands (see `docs/design/zero-config-imports.md`).

## Goal

A single project-wide config table in `pyproject.toml`, under `[tool.hyper]`, read by every consumer of `.hyper` files. `[tool.*]` is the standard namespace for tool config (ruff, mypy, pytest, hatch; Aerich uses `[tool.aerich]`). Hyper already ships `[tool.maturin]` and `[tool.pytest.ini_options]`, so the table fits.

```toml
[tool.hyper]
component_dirs = ["components"]   # dir names integrations scan
include = ["**/*.hyper"]          # what the import hook/CLI treat as components
html_validation = "strict"        # or "off"
```

## Why it's worth doing

Three consumers share zero config today:

1. The Rust CLI (`hyper generate`)
2. The IDE daemon (JetBrains)
3. The future import hook (`HyperFinder`, from zero-config-imports)

One `[tool.hyper]` table, read by the Rust core (the CLI and daemon already share `compile_to_python`), makes all three agree on what counts as a component and how it compiles. That shared source of truth is the real win, not just "config in a standard place."

## Build-time vs runtime split

Not all config belongs in pyproject.

| Config | Home | Why |
|---|---|---|
| What's a component, globs, HTML validation strictness | `[tool.hyper]` in pyproject | Build-time. Shared by CLI + daemon + import hook |
| Framework wiring (e.g. Django component dirs, bare-name resolution) | The framework's own idiom (Django settings, Jinja env) | That's where users of that framework look |

Don't fragment framework config into pyproject. Layer instead: a framework setting defaults to the pyproject value, which defaults to a built-in. For Django:

```
HYPER_COMPONENT_DIRS  (Django setting)
  └─ defaults to [tool.hyper].component_dirs
       └─ defaults to ["components"]
```

Settings override project defaults; project defaults override built-ins.

## Cost

- Python side: cheap. `tomllib` is stdlib (3.11+).
- Rust side: add the `toml` crate, plus walk-up-from-file root discovery to locate the right `pyproject.toml`. The daemon must resolve the correct pyproject per open file.

The Rust work belongs with the zero-config-imports milestone, since the import hook is the main new consumer. Adding `[tool.hyper]` before that would only feed the Django integration, which is better served by a Django setting in the meantime.

## References

- [PEP 518 — `pyproject.toml` build config](https://peps.python.org/pep-0518/)
- [PEP 621 — Project metadata in `pyproject.toml`](https://peps.python.org/pep-0621/)
- Precedents: ruff, mypy, pytest, hatch (`[tool.*]`); Aerich (`[tool.aerich]`)
