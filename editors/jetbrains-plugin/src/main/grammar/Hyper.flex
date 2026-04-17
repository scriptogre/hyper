package com.hyper.plugin.lexer;

import com.intellij.lexer.FlexLexer;
import com.intellij.psi.tree.IElementType;

import static com.intellij.psi.TokenType.BAD_CHARACTER;
import static com.intellij.psi.TokenType.WHITE_SPACE;
import static com.hyper.plugin.psi.HyperTypes.*;

%%

%{ 
  public _HyperLexer() {
    this((java.io.Reader)null);
  }
%}

%public
%class _HyperLexer
%implements FlexLexer
%function advance
%type IElementType
%unicode

EOL=\R
// We use lookahead or specific patterns to distinguish types
// Note: Flex matches longest match first, then order.

// Separator between frontmatter and body
SEPARATOR=[ \t]*"---"[ \t]*

// Comment lines starting with #
COMMENT_LINE=[ \t]*"#"[^\r\n]*

// Matches lines starting with optional whitespace then <
HTML_LINE=[ \t]*"<"[^\r\n]*

// Control flow: keyword + content + trailing colon (matching Rust transpiler's is_control_flow).
// The trailing `:` is required — without it, "for example, this is text" is content, not control flow.
// [^\r\n]*: backtracks to find the last `:` on the line, then allows optional whitespace and comment.
CONTROL_LINE_BODY=[ \t]*(async[ \t]+)?(if|for|while|match|def|class|elif|except|case|with|fragment)[ \t(][^\r\n]*:[ \t]*(#[^\r\n]*)?

// Bare block keywords: just keyword + colon (else:, try:, finally:, except:)
CONTROL_LINE_BARE=[ \t]*(else|try|finally|except)[ \t]*:[ \t]*(#[^\r\n]*)?

// Matches 'end' on a line by itself (with optional whitespace)
END_LINE=[ \t]*"end"[ \t]*

// Everything else is python
PYTHON_LINE=[^\r\n]+

%%

<YYINITIAL> {
  {SEPARATOR} {EOL}?     { return SEPARATOR_TOKEN; }
  {COMMENT_LINE} {EOL}?  { return COMMENT_TOKEN; }
  {HTML_LINE} {EOL}?     { return HTML_LINE_TOKEN; }
  {CONTROL_LINE_BODY} {EOL}?  { return CONTROL_LINE_TOKEN; }
  {CONTROL_LINE_BARE} {EOL}?  { return CONTROL_LINE_TOKEN; }
  {END_LINE} {EOL}?      { return END_LINE_TOKEN; }
  {PYTHON_LINE} {EOL}?   { return PYTHON_LINE_TOKEN; }
  {EOL}                  { return PYTHON_LINE_TOKEN; }
  [^]                    { return BAD_CHARACTER; }
}

