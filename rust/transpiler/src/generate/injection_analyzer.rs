//! Injection analyzer - computes IDE injection ranges from AST and generated code
//!
//! This is a separate concern from code generation, allowing the generator to stay clean
//! and focused on producing correct Python code.

use crate::ast::{Ast, Node, AttributeKind};
use super::output::{Range, RangeType, Injection, compute_injections};

/// Analyzes AST and generated code to produce injection ranges for IDE support
pub struct InjectionAnalyzer;

impl InjectionAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze AST and generated code to compute injection ranges and injections
    ///
    /// Note: This analyzer collects what SHOULD have injection ranges, but actual
    /// compiled positions must be tracked during generation. This method is now
    /// primarily used to post-process ranges that were collected during generation.
    pub fn analyze(&self, ast: &Ast, code: &str, ranges: Vec<Range>) -> (Vec<Range>, Vec<Injection>) {
        // Ranges are already tracked during generation with correct positions
        // Here we just compute injections from them
        let injections = compute_injections(code, &ranges);
        (ranges, injections)
    }

    /// Walk the AST and collect all nodes that need injection ranges
    fn collect_ranges(&self, ast: &Ast, ranges: &mut Vec<Range>) {
        for node in &ast.nodes {
            self.collect_node_ranges(node, ranges);
        }
    }

    /// Collect ranges for a single node
    fn collect_node_ranges(&self, node: &Node, ranges: &mut Vec<Range>) {
        match node {
            // Parameters in header section need Python injection
            Node::Parameter(param) => {
                ranges.push(Range {
                    range_type: RangeType::Python,
                    source_start: param.span.start.byte,
                    source_end: param.span.end.byte,
                    compiled_start: 0, // Parameters appear in function signature
                    compiled_end: 0,   // Exact position varies based on other params
                    needs_injection: false,
                });
            }

            // Expressions need Python injection
            Node::Expression(expr) => {
                ranges.push(Range {
                    range_type: RangeType::Python,
                    source_start: expr.span.start.byte,
                    source_end: expr.span.end.byte,
                    compiled_start: 0, // Will be computed by searching generated code
                    compiled_end: 0,
                    needs_injection: true,
                });
            }

            // Elements may contain expressions in attributes
            Node::Element(el) => {
                for attr in &el.attributes {
                    match &attr.kind {
                        AttributeKind::Dynamic { expr_span, .. } => {
                            ranges.push(Range {
                                range_type: RangeType::Python,
                                source_start: expr_span.start.byte,
                                source_end: expr_span.end.byte,
                                compiled_start: 0,
                                compiled_end: 0,
                                needs_injection: true,
                            });
                        }
                        AttributeKind::Shorthand { expr_span, .. } => {
                            ranges.push(Range {
                                range_type: RangeType::Python,
                                source_start: expr_span.start.byte,
                                source_end: expr_span.end.byte,
                                compiled_start: 0,
                                compiled_end: 0,
                                needs_injection: true,
                            });
                        }
                        AttributeKind::Spread { expr_span, .. } => {
                            ranges.push(Range {
                                range_type: RangeType::Python,
                                source_start: expr_span.start.byte,
                                source_end: expr_span.end.byte,
                                compiled_start: 0,
                                compiled_end: 0,
                                needs_injection: true,
                            });
                        }
                        _ => {}
                    }
                }

                // Recurse into children
                for child in &el.children {
                    self.collect_node_ranges(child, ranges);
                }
            }

            // Control flow nodes - recurse into bodies
            Node::If(if_node) => {
                for child in &if_node.then_branch {
                    self.collect_node_ranges(child, ranges);
                }
                for (_cond, _span, body) in &if_node.elif_branches {
                    for child in body {
                        self.collect_node_ranges(child, ranges);
                    }
                }
                if let Some(else_branch) = &if_node.else_branch {
                    for child in else_branch {
                        self.collect_node_ranges(child, ranges);
                    }
                }
            }

            Node::For(for_node) => {
                for child in &for_node.body {
                    self.collect_node_ranges(child, ranges);
                }
            }

            Node::Match(match_node) => {
                for case in &match_node.cases {
                    for child in &case.body {
                        self.collect_node_ranges(child, ranges);
                    }
                }
            }

            Node::While(while_node) => {
                for child in &while_node.body {
                    self.collect_node_ranges(child, ranges);
                }
            }

            Node::With(with_node) => {
                for child in &with_node.body {
                    self.collect_node_ranges(child, ranges);
                }
            }

            Node::Try(try_node) => {
                for child in &try_node.body {
                    self.collect_node_ranges(child, ranges);
                }
                for except_clause in &try_node.except_clauses {
                    for child in &except_clause.body {
                        self.collect_node_ranges(child, ranges);
                    }
                }
                if let Some(else_clause) = &try_node.else_clause {
                    for child in else_clause {
                        self.collect_node_ranges(child, ranges);
                    }
                }
                if let Some(finally_clause) = &try_node.finally_clause {
                    for child in finally_clause {
                        self.collect_node_ranges(child, ranges);
                    }
                }
            }

            Node::Component(comp) => {
                // Recurse into component children
                for child in &comp.children {
                    self.collect_node_ranges(child, ranges);
                }
            }

            Node::Fragment(frag) => {
                for child in &frag.children {
                    self.collect_node_ranges(child, ranges);
                }
            }

            // Other nodes don't need injection ranges
            _ => {}
        }
    }
}
