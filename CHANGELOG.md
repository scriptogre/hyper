# Changelog

## 0.2.1 (2026-09-21)

- Expose colored compiler diagnostics through the Python API

## 0.2.0 (2026-09-21)

- Replace `component` declarations with `def ... -> Component`
- Bind component props lazily and render with `.render()`
- Stream with `.render(stream=True)`
- Export nested components explicitly with `@subcomponent`
- Make aligned `end` markers optional
- Expose the Python runtime as `hyper`
- Expose generated Python through standard inspection

## 0.1.2

- Fix named slot composition between Hyper components
- Document explicit components and current integration imports

## 0.1.1

- Include the license in the source distribution
- Check source metadata paths before publishing

## 0.1.0

Initial alpha release.

- Import `.hyper` files without a build step or generated Python
- Compose callable components with typed props, slots, and nested components
- Stream directly from each component's generated function
- Use Python control flow, imports, functions, classes, and async code
- Write multiline HTML and component tags
- Render components through FastAPI, Django, and Jinja
- Load Markdown, JSON, YAML, and TOML content collections
- Edit with JetBrains, TextMate, and VS Code support
