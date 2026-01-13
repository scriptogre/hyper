use std::collections::HashMap;
use std::sync::Arc;

// Re-export Position and Span from tokenizer to avoid duplication
// This allows the rest of the codebase to use a single Span type
pub use crate::parser::tokenizer::{Position, Span};

/// Abstract Syntax Tree
#[derive(Debug, Clone)]
pub struct Ast {
    pub nodes: Vec<Node>,
    pub source: Arc<str>,
}

impl Ast {
    pub fn new(nodes: Vec<Node>, source: Arc<str>) -> Self {
        Self { nodes, source }
    }
}

/// AST Node
#[derive(Debug, Clone)]
pub enum Node {
    // Content
    Text(TextNode),
    Expression(ExpressionNode),

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
    pub span: Span,
}

/// Python expression
#[derive(Debug, Clone)]
pub struct ExpressionNode {
    pub expr: String,
    pub span: Span,
    pub escape: bool, // true = escape HTML, false = raw
}

/// HTML element
#[derive(Debug, Clone)]
pub struct ElementNode {
    pub tag: String,
    pub tag_span: Span,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub self_closing: bool,
    pub span: Span,
}

/// Component invocation
#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub name: String,
    pub name_span: Span,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub slots: HashMap<String, Vec<Node>>,
    pub span: Span,
}

/// Fragment (bare children without wrapper)
#[derive(Debug, Clone)]
pub struct FragmentNode {
    pub children: Vec<Node>,
    pub span: Span,
}

/// Slot placeholder
#[derive(Debug, Clone)]
pub struct SlotNode {
    pub name: Option<String>,
    pub fallback: Vec<Node>,
    pub span: Span,
}

/// If/elif/else
#[derive(Debug, Clone)]
pub struct IfNode {
    pub condition: String,
    pub condition_span: Span,
    pub then_branch: Vec<Node>,
    pub elif_branches: Vec<(String, Span, Vec<Node>)>,
    pub else_branch: Option<Vec<Node>>,
    pub span: Span,
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForNode {
    pub binding: String,      // "item" or "i, item"
    pub iterable: String,     // The Python expression
    pub iterable_span: Span,
    pub body: Vec<Node>,
    pub is_async: bool,       // async for
    pub span: Span,
}

/// Match/case
#[derive(Debug, Clone)]
pub struct MatchNode {
    pub expr: String,
    pub expr_span: Span,
    pub cases: Vec<CaseNode>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseNode {
    pub pattern: String,
    pub pattern_span: Span,
    pub body: Vec<Node>,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileNode {
    pub condition: String,
    pub condition_span: Span,
    pub body: Vec<Node>,
    pub span: Span,
}

/// With statement (context manager)
#[derive(Debug, Clone)]
pub struct WithNode {
    pub items: String, // "open(file) as f" or "lock, other"
    pub items_span: Span,
    pub body: Vec<Node>,
    pub is_async: bool, // async with
    pub span: Span,
}

/// Try/except/else/finally
#[derive(Debug, Clone)]
pub struct TryNode {
    pub body: Vec<Node>,
    pub except_clauses: Vec<ExceptClause>,
    pub else_clause: Option<Vec<Node>>,
    pub finally_clause: Option<Vec<Node>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExceptClause {
    pub exception: Option<String>, // None for bare "except:"
    pub exception_span: Option<Span>,
    pub body: Vec<Node>,
    pub span: Span,
}

/// Python statement (assignment, expression statement, etc.)
#[derive(Debug, Clone)]
pub struct StatementNode {
    pub stmt: String,
    pub span: Span,
}

/// Function or class definition
#[derive(Debug, Clone)]
pub struct DefinitionNode {
    pub kind: DefinitionKind,
    pub signature: String, // "def foo(x: int):" or "class Foo:"
    pub signature_span: Span,
    pub body: Vec<Node>,
    pub span: Span,
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
    pub span: Span,
}

/// Template parameter (in header)
#[derive(Debug, Clone)]
pub struct ParameterNode {
    pub name: String,
    pub type_hint: Option<String>,
    pub default: Option<String>,
    pub span: Span,
}

/// Decorator
#[derive(Debug, Clone)]
pub struct DecoratorNode {
    pub decorator: String, // "@fragment" or "@app.route('/path')"
    pub span: Span,
}

/// Attribute on element or component
#[derive(Debug, Clone)]
pub struct Attribute {
    pub kind: AttributeKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AttributeKind {
    /// Static: class="foo"
    Static {
        name: String,
        value: String,
    },

    /// Dynamic: class={expr}
    Dynamic {
        name: String,
        expr: String,
        expr_span: Span,
    },

    /// Boolean: disabled
    Boolean {
        name: String,
    },

    /// Shorthand: {disabled}
    Shorthand {
        name: String,
        expr_span: Span,
    },

    /// Spread: {...props}
    Spread {
        expr: String,
        expr_span: Span,
    },

    /// Slot assignment: slot:name or slot:name={expr}
    SlotAssignment {
        name: String,
        expr: Option<String>,
        expr_span: Option<Span>,
    },
}