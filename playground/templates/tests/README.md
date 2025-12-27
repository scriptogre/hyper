# Hyper Plugin Test Templates

Each `.hyper` file is a self-contained component with its own props signature.

## Basic Components
- `BasicCard.hyper` - Simplest component
- `CardWithDefaults.hyper` - Default parameter values
- `ComplexTypes.hyper` - Complex type annotations
- `UserProfile.hyper` - Nested dict access

## Lists and Loops
- `ItemList.hyper` - Basic for loop with enumerate
- `NestedLoops.hyper` - Table with nested loops
- `WhileLoop.hyper` - While loop patterns

## Conditionals
- `ConditionalChain.hyper` - Full if/elif/else
- `MatchCase.hyper` - Pattern matching
- `MatchCaseGuards.hyper` - Match with guard conditions

## Error Handling
- `TryExcept.hyper` - Exception handling
- `WithStatement.hyper` - Context managers

## Async
- `AsyncFetch.hyper` - await triggers async def
- `AsyncStream.hyper` - async for/with

## Functions
- `FunctionDef.hyper` - Helper functions in template
- `RecursiveTree.hyper` - Recursive rendering

## HTML Patterns
- `HtmlAttributes.hyper` - Complex attributes
- `SelfClosingTags.hyper` - Void elements
- `SvgContent.hyper` - SVG in templates
- `Expressions.hyper` - Complex {expressions}

## Composition
- `SlotPattern.hyper` - Children slot with {...}
- `ConditionalWrapper.hyper` - Conditional wrapping
- `DeepNesting.hyper` - Many levels deep

## UI Patterns
- `LoadingState.hyper` - Loading/error/content
- `FormField.hyper` - Reusable form field
- `Pagination.hyper` - Page navigation
- `Tabs.hyper` - Tab component
- `Modal.hyper` - Dialog overlay

## Lexer Edge Cases
- `LexerEdge_Comparison.hyper` - `<` vs HTML
- `LexerEdge_Keywords.hyper` - keyword-like identifiers
- `LexerEdge_End.hyper` - `end` keyword edge cases

## Generator Edge Cases
- `GeneratorEdge_Empty.hyper` - No parameters
- `GeneratorEdge_ParamsOnly.hyper` - Params, no body
- `GeneratorEdge_Comments.hyper` - Comment positions
- `GeneratorEdge_Unicode.hyper` - Unicode offset mapping
- `GeneratorEdge_LongLines.hyper` - Very long lines
- `GeneratorEdge_TripleQuotes.hyper` - Triple quotes in body
- `GeneratorEdge_EmptyBlocks.hyper` - Empty control blocks
