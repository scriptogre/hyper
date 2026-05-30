# Zero-Config Imports

**Status:** Approved design, not yet implemented.

## Goal

Eliminate the build step. `.hyper` files compile transparently on import. No intermediate `.py` files on disk, no `hyper generate` command, no manual build invocation.

Before:

```
Sidebar.hyper  ──hyper generate──▶  Sidebar.py  ──import──▶  Sidebar
```

After:

```
Sidebar.hyper  ──import──▶  Sidebar
```

## User-facing API

Two import forms, one for each `.hyper` file pattern.

**Short form** — for single-component files. The file stem matches the component function name.

```python
# Sidebar.hyper defines Sidebar.
from app.components import Sidebar
```

**Long form** — for library files (multiple top-level `def`s in one `.hyper`, no `---` separator; see `docs/design/templates.md`) and any explicit file-level import.

```python
# widgets.hyper defines Header and Footer.
from app.components.widgets import Header, Footer
```

The short form is sugar for the file-stem-matches-component case. The long form is the base case: every `.hyper` file is a module. For single-component files both forms work; short is the documented idiom.

## Mechanism

Three pieces. No literal code in this doc — implementation details go in the code itself.

**`HyperFinder`** — a `MetaPathFinder` inserted at `sys.meta_path[0]`. For any import name, walks the parent's `__path__` (or `sys.path` for top-level lookups) and:

- Claims any directory containing `.hyper` files (or whose subdirectories do, recursively) as a package. The returned spec has `is_package=True` and `submodule_search_locations` set so Python's normal submodule resolution still walks into subdirectories.
- Claims any matching `.hyper` file as a module.

**`HyperPackageLoader`** — handles directories. `exec_module` runs the directory's `__init__.py` if present, then installs a module-level `__getattr__` (PEP 562) on the module. The `__getattr__` looks for `{name}.hyper` in the directory; if found, compiles it via PyO3, execs the result in a fresh namespace, extracts the `{name}` symbol, caches it on the package module, returns it. This is the short-form path. If `{name}.hyper` doesn't exist or doesn't define `{name}`, raises `AttributeError`.

**`HyperModuleLoader`** — handles individual `.hyper` files. Compiles the file via PyO3 and execs the result into the module's `__dict__`. Standard Python module — every top-level `def`, `class`, and `import` is accessible. This is the long-form path; library files always go through it.

The `__init__.py` running first means user-defined attributes shadow short-form `.hyper` lookups (explicit user code wins). Dropping an empty `__init__.py` into a `.hyper` directory does **not** disable component imports — `__getattr__` is still installed.

Submodule imports (`from app.components.forms import Login` where `forms/` is a subdirectory) are handled by Python's normal submodule machinery — `HyperFinder` claims the subdirectory the same way it claims its parent.

## Rust binding

The Rust transpiler is exposed to Python via PyO3 as `hyper._native.transpile(source, filename) -> str`. In-process call. No subprocess fallback — if the native module fails to import, Hyper is broken and surfaces a clear `ImportError`.

Extract `compile_to_python(source, filename) -> Result<String, ParseError>` as a public function in `rust/src/lib.rs` so the daemon binary and the PyO3 binding share the same compile path.

## Install

Ship `hyper.pth` in the wheel containing a single import:

```
import hyper._autohook
```

`hyper/_autohook.py` calls `_install_finder()` as a side effect. Same mechanism `coverage.py` and `pyximport` use. Microseconds at interpreter startup; zero ongoing cost for processes that never touch `.hyper` files.

Tests call `hyper._autohook._install_finder()` / `_uninstall_finder()` directly for setup/teardown.

## Concurrency and caching

Per-module lock around the `__getattr__` compile path — two threads requesting the same component compile once. Locks are per-module so component composition (a `.hyper` whose compiled output imports another) uses different locks and cannot deadlock.

In-memory cache, process lifetime. No on-disk cache. No mtime invalidation. Hot reload is deferred (see below).

## `hyper generate` removal

The `hyper generate` subcommand is deleted from `rust/src/main.rs`. The Rust binary keeps `daemon` mode (used by the JetBrains plugin's existing daemon protocol).

Everything previously documented as "run `hyper generate`" becomes "just import." That includes:

- README quick start
- `docs/design/templates.md`, `docs/implementation/templates.md`
- `python/tests/integrations/conftest.py` (subprocess call → `_install_finder()`)
- Any examples with pre-commit / CI hooks

## Files

**Created**
- `python/hyper/_loader.py` — `HyperFinder`, `HyperPackageLoader`, `HyperModuleLoader`
- `python/hyper/_autohook.py` — `_install_finder()`, `_uninstall_finder()`, side-effect on import
- `python/hyper.pth` — one line: `import hyper._autohook`
- `python/tests/test_loader.py` — unit tests
- `python/tests/fixtures/zero_config/` — end-to-end fixture project tree
- `rust/src/python_module.rs` — PyO3 wrapper exposing `transpile()`

**Modified**
- `rust/src/lib.rs` — extract `compile_to_python(source, filename)` shared by daemon + PyO3
- `rust/Cargo.toml` — add `pyo3` dependency
- `pyproject.toml` — declare the PyO3 module target
- `python/tests/integrations/conftest.py` — install hook instead of subprocess
- README, `docs/design/templates.md`, `docs/implementation/templates.md` — rewrite for the new flow

**Deleted**
- `rust/src/main.rs` `generate` subcommand (keep `daemon`)

## Tests

**Unit** (`python/tests/test_loader.py`)
- Finder claims directories with `.hyper` files and individual `.hyper` files; returns `None` for unrelated names
- Short form: `from pkg import Sidebar` resolves `Sidebar.hyper`'s `Sidebar` symbol via `__getattr__`
- Long form: `from pkg.Sidebar import Sidebar` loads `Sidebar.hyper` as a module
- Library file: `from pkg.widgets import Header, Footer` exposes both top-level functions
- `__getattr__` compiles on first access, hits cache on second
- Compile errors surface as `ImportError` with `ParseError` chained
- Two threads importing the same component concurrently compile once
- `_install_finder()` is idempotent; `_uninstall_finder()` removes cleanly
- User-provided `__init__.py` runs and its attributes shadow `.hyper` lookups
- Empty `__init__.py` does not break `.hyper` imports
- Component-to-component imports (`from .Card import Card` inside compiled `.hyper` output) resolve through the finder

**End-to-end** (`python/tests/fixtures/zero_config/`)
- Real project tree: `components/Greeting.hyper`, `components/Card.hyper`, no `.py`, no `__init__.py`
- Subprocess runs an app that imports a component, asserts rendered HTML
- Asserts no `.py` files were written during the run

**Integration** (existing `python/tests/integrations/`)
- Jinja2 / Django tests pass unchanged after conftest swaps `hyper generate` subprocess for `_install_finder()`

## Real risks

1. **`.pth` packaging through maturin.** Verify `python/hyper.pth` lands in the site-packages root for both wheel install (`pip install`) and editable install (`pip install -e .` / `maturin develop`). The two paths handle `.pth` files differently.

2. **PyO3 import failure visibility.** If `hyper._native` fails to import (broken wheel, exotic platform), the `.pth` import fails silently per Python's `.pth` semantics. `_autohook` should catch the import error and emit a clear `warnings.warn()` so users see what's wrong.

3. **Subprocess-spawned Python processes.** `pytest-xdist` workers, `debugpy`, `multiprocessing` spawn fresh interpreters that re-run `.pth` files. Should "just work" but worth a smoke test.

## Deferred

- Hot reload / file watching
- Traceback rewriting (map runtime errors back to `.hyper` source lines via `linecache`)
- Cross-editor LSP for non-JetBrains tools
- JetBrains plugin extension that teaches the Python plugin about the virtual namespace (`PyTypeProvider`, `PyReferenceContributor`)
- On-disk compile cache (only if cold-start cost becomes a real problem)

## References

- [PEP 562 — Module `__getattr__` and `__dir__`](https://peps.python.org/pep-0562/)
- [PEP 420 — Namespace packages](https://peps.python.org/pep-0420/)
- [`importlib` API](https://docs.python.org/3/library/importlib.html)
- [`.pth` mechanism](https://docs.python.org/3/library/site.html)
- [PyO3](https://pyo3.rs/) / [maturin](https://www.maturin.rs/)
- Precedents: `coverage.py`, `pyximport` (Cython), `editables`
