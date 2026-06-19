//! The template AST: parsed from `.hyper`, lowered in place by the plugin
//! passes (so it doubles as the compiler's IR), then printed by the
//! generator. HTML nodes (Element, Text, Component, Slot) are Hyper-specific;
//! the Python parts converge toward ruff's AST. The generated Python is a
//! separate tree.

use std::collections::HashMap;
use std::sync::Arc;

// Re-export Position and TextRange from tokenizer to avoid duplication
// This allows the rest of the codebase to use a single TextRange type
pub use crate::parse::tokenizer::{Position, TextRange};

/// Abstract Syntax Tree
#[derive(Debug, Clone)]
pub struct Ast {
    pub function: Function,
    pub source: Arc<str>,
}

impl Ast {
    pub fn new(function: Function, source: Arc<str>) -> Self {
        Self { function, source }
    }
}

/// The template's top-level function, with frontmatter split from body by the
/// `lower` pass. `params` and `body` hold `Node`s so plugins can walk them;
/// the other frontmatter buckets are typed since no plugin visits them.
#[derive(Debug, Clone)]
pub struct Function {
    pub is_async: bool,
    pub params: Vec<Node>,
    pub imports: Vec<ImportNode>,
    pub decorators: Vec<DecoratorNode>,
    pub header_comments: Vec<CommentNode>,
    pub body: Vec<Node>,
}

/// AST Node
#[derive(Debug, Clone)]
pub enum Node {
    // Content
    Text(TextNode),
    Expression(ExpressionNode),
    Comment(CommentNode),

    // Structure
    Element(ElementNode),
    Component(ComponentNode),
    Fragment(FragmentNode),
    Slot(SlotNode),

    // Control Flow
    If(IfNode),
    For(ForNode),
    Match(MatchNode),
    While(WhileNode),
    With(WithNode),
    Try(TryNode),

    // Python
    Statement(StatementNode),
    Definition(DefinitionNode),
    Import(ImportNode),
    Parameter(ParameterNode),
    Decorator(DecoratorNode),
}

/// Text content (HTML, whitespace, etc.)
#[derive(Debug, Clone)]
pub struct TextNode {
    pub content: String,
    pub range: TextRange,
}

/// Comment (Python-style # comment)
#[derive(Debug, Clone)]
pub struct CommentNode {
    pub text: String, // includes the # prefix
    pub range: TextRange,
    pub inline: bool, // true if comment follows content on the same source line
}

/// Python expression
#[derive(Debug, Clone)]
pub struct ExpressionNode {
    pub expr: String,
    pub range: TextRange,
    pub escape: bool,                // true = escape HTML, false = raw
    pub format_spec: Option<String>, // e.g. "03d", ".2f", ">20"
    pub conversion: Option<char>,    // 'r', 's', or 'a'
    pub debug: bool,                 // true if {value=}
}

/// HTML element
#[derive(Debug, Clone)]
pub struct ElementNode {
    pub tag: String,
    pub tag_range: TextRange,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub self_closing: bool,
    pub range: TextRange,
    pub close_range: Option<TextRange>, // TextRange of </tag> closing tag in source
}

/// Component invocation
#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub name: String,
    pub name_range: TextRange,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub slots: HashMap<String, Vec<Node>>,
    pub range: TextRange,
    pub close_range: Option<TextRange>,
}

/// Fragment (bare children without wrapper)
#[derive(Debug, Clone)]
pub struct FragmentNode {
    pub children: Vec<Node>,
    pub range: TextRange,
}

/// Slot placeholder
#[derive(Debug, Clone)]
pub struct SlotNode {
    pub name: Option<String>,
    pub fallback: Vec<Node>,
    pub range: TextRange,
    pub close_range: Option<TextRange>,
}

/// If/elif/else
#[derive(Debug, Clone)]
pub struct IfNode {
    pub condition: String,
    pub condition_range: TextRange,
    pub then_branch: Vec<Node>,
    pub elif_branches: Vec<(String, TextRange, Vec<Node>)>,
    pub else_branch: Option<Vec<Node>>,
    pub range: TextRange,
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForNode {
    pub binding: String, // "item" or "i, item"
    pub binding_range: TextRange,
    pub iterable: String, // The Python expression
    pub iterable_range: TextRange,
    pub body: Vec<Node>,
    pub is_async: bool, // async for
    pub range: TextRange,
}

/// Match/case
#[derive(Debug, Clone)]
pub struct MatchNode {
    pub expr: String,
    pub expr_range: TextRange,
    pub cases: Vec<CaseNode>,
    pub range: TextRange,
}

#[derive(Debug, Clone)]
pub struct CaseNode {
    pub pattern: String,
    pub pattern_range: TextRange,
    pub body: Vec<Node>,
    pub range: TextRange,
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileNode {
    pub condition: String,
    pub condition_range: TextRange,
    pub body: Vec<Node>,
    pub range: TextRange,
}

/// With statement (context manager)
#[derive(Debug, Clone)]
pub struct WithNode {
    pub items: String, // "open(file) as f" or "lock, other"
    pub items_range: TextRange,
    pub body: Vec<Node>,
    pub is_async: bool, // async with
    pub range: TextRange,
}

/// Try/except/else/finally
#[derive(Debug, Clone)]
pub struct TryNode {
    pub body: Vec<Node>,
    pub except_clauses: Vec<ExceptClause>,
    pub else_clause: Option<Vec<Node>>,
    pub finally_clause: Option<Vec<Node>>,
    pub range: TextRange,
}

#[derive(Debug, Clone)]
pub struct ExceptClause {
    pub exception: Option<String>, // None for bare "except:"
    pub exception_range: Option<TextRange>,
    pub body: Vec<Node>,
    pub range: TextRange,
}

/// Python statement (assignment, expression statement, etc.)
#[derive(Debug, Clone)]
pub struct StatementNode {
    pub stmt: String,
    pub range: TextRange,
}

/// Function or class definition
#[derive(Debug, Clone)]
pub struct DefinitionNode {
    pub kind: DefinitionKind,
    pub signature: String, // "def foo(x: int):" or "class Foo:"
    pub signature_range: TextRange,
    pub body: Vec<Node>,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionKind {
    Function,
    Class,
}

/// Import statement
#[derive(Debug, Clone)]
pub struct ImportNode {
    pub stmt: String, // "import foo" or "from foo import bar"
    pub range: TextRange,
}

/// Where a parameter sits in the function signature (mirrors Python's argument
/// categories: `args` / `kwonlyargs` / `kwarg`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// Positional-or-keyword, before the `*` marker (e.g. the default slot).
    Positional,
    /// Keyword-only, after the `*` marker (user params, named slots).
    KeywordOnly,
    /// `**kwargs`.
    VarKeyword,
}

/// Template parameter (in header)
#[derive(Debug, Clone)]
pub struct ParameterNode {
    pub name: String,
    pub type_hint: Option<String>,
    pub default: Option<String>,
    pub kind: ParamKind,
    pub range: TextRange,
}

/// Decorator
#[derive(Debug, Clone)]
pub struct DecoratorNode {
    pub decorator: String, // "@fragment" or "@app.route('/path')"
    pub range: TextRange,
}

/// Attribute on element or component
#[derive(Debug, Clone)]
pub struct Attribute {
    pub kind: AttributeKind,
    pub range: TextRange,
}

#[derive(Debug, Clone)]
pub enum AttributeKind {
    /// Static: class="foo"
    Static { name: String, value: String },

    /// Expression: class={expr}
    Expression {
        name: String,
        expr: String,
        expr_range: TextRange,
    },

    /// Template: class="{expr} static" (mixed expressions in quoted value)
    Template {
        name: String,
        value: String, // Raw value with {expr} markers
    },

    /// Boolean: disabled
    Boolean { name: String },

    /// Shorthand: {disabled} — emits name=name
    Shorthand { name: String, expr_range: TextRange },

    /// Spread: {**props} — kwargs unpacking
    Spread { expr: String, expr_range: TextRange },

    /// Slot assignment: {...name}
    SlotAssignment {
        name: String,
        expr: Option<String>,
        expr_range: Option<TextRange>,
    },
}
