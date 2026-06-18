//! Phase 3: transform passes that operate directly on the lowered Ruff
//! `ModModule`, replacing the old `Context` side-channel.
//!
//! Each pass mutates the Python AST in place. Unlike the old plugins, they read
//! their input from the real lowered code (e.g. which helper functions are
//! actually called) rather than from a parallel analysis of the hyper AST — this
//! is what lets the printer stay mechanical.

use std::collections::HashSet;

use ruff_python_ast::visitor::{self, Visitor};
use ruff_python_ast::{self as ast, Expr, Stmt};

use super::builders as b;

/// Runtime helpers importable from `hyper`, in canonical import order.
const HELPERS: &[&str] = &[
    "escape",
    "safe",
    "render_class",
    "render_style",
    "render_attr",
    "render_data",
    "render_aria",
    "spread_attrs",
];

/// Collects the names of helper functions referenced anywhere in the module.
struct HelperCollector {
    used: HashSet<&'static str>,
}

impl<'a> Visitor<'a> for HelperCollector {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Name(name) = expr {
            if let Some(helper) = HELPERS.iter().find(|h| **h == name.id.as_str()) {
                self.used.insert(helper);
            }
        }
        visitor::walk_expr(self, expr);
    }
}

/// Rewrite the generated `from hyper import html` statement so it imports exactly
/// the helpers the lowered body uses (`escape`, `render_class`, …), in canonical
/// order. Because detection runs on the real lowered code, helpers that are not
/// actually emitted are not imported (no dead imports).
pub fn apply_helper_imports(module: &mut ast::ModModule) {
    let mut collector = HelperCollector {
        used: HashSet::new(),
    };
    for stmt in &module.body {
        collector.visit_stmt(stmt);
    }

    let mut names: Vec<(&str, Option<&str>)> = vec![("html", None)];
    for helper in HELPERS {
        if collector.used.contains(helper) {
            names.push((helper, None));
        }
    }

    if let Some(import) = find_hyper_import(&mut module.body) {
        *import = b::import_from("hyper", &names, b::SENTINEL);
    }
}

/// Detects `await` / `async for` / `async with` usage within a scope, without
/// descending into nested function or class definitions (which form their own
/// async scope).
struct AsyncDetector {
    is_async: bool,
}

impl<'a> Visitor<'a> for AsyncDetector {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            // A nested def/class is its own scope; its async-ness is independent.
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => {}
            Stmt::For(f) if f.is_async => {
                self.is_async = true;
                visitor::walk_stmt(self, stmt);
            }
            Stmt::With(w) if w.is_async => {
                self.is_async = true;
                visitor::walk_stmt(self, stmt);
            }
            _ => visitor::walk_stmt(self, stmt),
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if matches!(expr, Expr::Await(_)) {
            self.is_async = true;
        }
        visitor::walk_expr(self, expr);
    }
}

/// Mark the outer template function `async` when its body uses `await`,
/// `async for`, or `async with`.
pub fn apply_async(module: &mut ast::ModModule) {
    if let Some(Stmt::FunctionDef(func)) = module.body.last_mut() {
        let mut detector = AsyncDetector { is_async: false };
        for stmt in &func.body {
            detector.visit_stmt(stmt);
        }
        if detector.is_async {
            func.is_async = true;
        }
    }
}

/// Find the generated `from hyper import …` statement (the one the lowering
/// inserts), so a pass can rewrite its imported names.
fn find_hyper_import(body: &mut [Stmt]) -> Option<&mut Stmt> {
    body.iter_mut().find(|stmt| {
        matches!(
            stmt,
            Stmt::ImportFrom(ast::StmtImportFrom { module: Some(m), names, .. })
                if m.as_str() == "hyper" && names.iter().any(|a| a.name.as_str() == "html")
        )
    })
}
